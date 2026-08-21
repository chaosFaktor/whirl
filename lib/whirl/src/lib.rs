#![feature(associated_type_defaults, trait_alias, lazy_type_alias)]
pub mod error;
#[cfg(test)]
pub mod test;
use std::{
    any::Any,
    convert::Infallible,
    pin::{Pin, pin},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use pollster::FutureExt;
use tokio::{
    sync::{
        mpsc::{
            self,
            error::{SendError, SendTimeoutError, TrySendError},
        },
        oneshot,
    },
    task::{JoinError, coop::RestoreOnPending},
};

use crate::error::{AnyError, CallError, CallTimeoutError, EnvelopeProcessError};

mod actor_registry;
pub use actor_registry::ActorRegistry;

#[derive(Debug)]
pub enum ActorRunningState {
    Running,
    Panicked,
    Cancelled,
}
struct ActorTaskJoinHandle {
    join_handle: Option<tokio::task::JoinHandle<()>>,
    join_error: Option<JoinError>,
}
impl ActorTaskJoinHandle {
    pub fn new(join_handle: tokio::task::JoinHandle<()>) -> Self {
        Self {
            join_handle: Some(join_handle),
            join_error: None,
        }
    }
    pub fn is_running(&self) -> bool {
        if let Some(join_handle) = &self.join_handle {
            return !join_handle.is_finished();
        }
        false
    }
    fn try_get_join_error(&mut self) -> Option<&JoinError> {
        if self.join_error.is_some() {
            return self.join_error.as_ref();
        }
        if self.is_running() {
            return None;
        }
        if let Some(join_handle) = self.join_handle.take() {
            let res = join_handle.block_on();

            self.join_error = res.err();
            return self.join_error.as_ref();
        }

        None
    }
    pub fn running_state(&mut self) -> ActorRunningState {
        if let Some(join_error) = self.try_get_join_error() {
            if join_error.is_panic() {
                return ActorRunningState::Panicked;
            } else if join_error.is_cancelled() {
                return ActorRunningState::Cancelled;
            }
        }
        return ActorRunningState::Running;
    }
    pub fn kill(&self) {
        if let Some(join_handle) = &self.join_handle {
            join_handle.abort();
        }
    }
}
pub struct ActorAddr<A: ActorMarker + ?Sized> {
    tx: mpsc::Sender<Box<dyn ProcessableMessage<A>>>,
    join_handle: Arc<tokio::sync::RwLock<ActorTaskJoinHandle>>,
}
impl<H: ActorMarker> ActorAddr<H> {
    fn new(
        tx: mpsc::Sender<Box<dyn ProcessableMessage<H>>>,
        join_handle: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            tx,
            join_handle: Arc::new(tokio::sync::RwLock::new(ActorTaskJoinHandle::new(
                join_handle,
            ))),
        }
    }
    pub fn kill(&self) {
        let join_handle = self.join_handle.clone();
        tokio::task::spawn(async move {
            let lock = join_handle.read().await;
            lock.kill();
        });
    }
    pub async fn running_state(&self) -> ActorRunningState {
        let mut lock = self.join_handle.write().await;
        lock.running_state()
    }

    pub async fn wait_cast<M: Message>(
        &self,
        msg: M,
    ) -> Result<(), SendError<Box<dyn ProcessableMessage<H>>>>
    where
        H: Handle<M>,
    {
        let envelope = Envelope::new_without_reply(msg);
        self.tx.send(envelope).await?;
        Ok(())
    }
    pub fn cast<M: Message>(
        &self,
        msg: M,
    ) -> Result<(), TrySendError<Box<dyn ProcessableMessage<H>>>>
    where
        H: Handle<M>,
    {
        let envelope = Envelope::new_without_reply(msg);
        self.tx.try_send(envelope)?;
        Ok(())
    }
    pub async fn cast_timeout<M: Message>(
        &self,
        msg: M,
        timeout: Duration,
    ) -> Result<(), SendTimeoutError<Box<dyn ProcessableMessage<H>>>>
    where
        H: Handle<M>,
    {
        let envelope = Envelope::new_without_reply(msg);
        self.tx.send_timeout(envelope, timeout).await?;
        Ok(())
    }
    pub async fn call<M: Message>(&self, msg: M) -> Result<H::Reply, CallError<M, H>>
    where
        H: Handle<M>,
    {
        let (envelope, rx) = Envelope::new_with_reply(msg);
        self.tx.send(envelope).await?;
        Ok(rx.await?)
    }
    pub async fn call_timeout<M: Message>(
        &self,
        msg: M,
        timeout: Duration,
    ) -> Result<H::Reply, CallTimeoutError<M, H>>
    where
        H: Handle<M>,
    {
        let (envelope, rx) = Envelope::new_with_reply(msg);
        self.tx.send_timeout(envelope, timeout).await?;
        let response = tokio::time::timeout(timeout, rx).await??;
        Ok(response)
    }
}
unsafe impl<A: ActorMarker> Send for ActorAddr<A> {}

pub trait ActorMarker: 'static + Send {}
pub trait ActorError: 'static {
    type ErrorHandler: ErrorHandler<Self::ActorError> = DefaultErrorHandler;
    type ActorError: std::error::Error = AnyError;
}
pub trait Actor: ActorMarker + ActorError + Sized {
    fn start(mut self, mailbox_size: usize) -> ActorAddr<Self> {
        let (tx, mut rx) = mpsc::channel::<Box<dyn ProcessableMessage<Self>>>(mailbox_size);
        let join_handle = tokio::spawn(async move {
            Self::on_spawn();
            while let Some(msg) = rx.recv().await {
                let res = msg.process(&mut self).await;
                if let Err(error) = res {
                    Self::ErrorHandler::process_error(error);
                }
            }
        });
        let handle = ActorAddr::new(tx, join_handle);
        handle
    }
    /// run inside thread when spawned
    fn on_spawn() {}
}

pub trait ErrorHandler<E: std::error::Error> {
    fn process_error(error: E);
    fn install_panic_handler();
}
pub struct PanickingErrorHandler;
impl<E: std::error::Error> ErrorHandler<E> for PanickingErrorHandler {
    fn process_error(error: E) {
        Err::<(), _>(error).unwrap();
    }
    fn install_panic_handler() {}
}

pub type DefaultErrorHandler = PanickingErrorHandler;

#[async_trait]
pub trait Message: Send + 'static {}

#[async_trait]
/// Implement *handle_fallibly* or *handle_infallibly*
pub trait Handle<M: Message>: ActorMarker + ActorError {
    type Reply: Message;
    type HandleError: Into<Self::ActorError> = Self::ActorError;
    async fn handle_fallibly(&mut self, msg: M) -> Result<Self::Reply, Self::HandleError> {
        Ok(self.handle_infallibly(msg).await)
    }
    async fn handle_infallibly(&mut self, msg: M) -> Self::Reply {
        let _ = msg;
        todo!()
    }
}

#[async_trait]
pub trait ProcessableMessage<A: ActorMarker + ActorError>: Send {
    async fn process(self: Box<Self>, actor: &mut A) -> Result<(), A::ActorError>;
}
pub struct Envelope<M: Message, R: Message> {
    msg: M,
    tx_call: Option<oneshot::Sender<R>>,
}
impl<M: Message, R: Message> Envelope<M, R> {
    pub fn new_with_reply(msg: M) -> (Box<Self>, oneshot::Receiver<R>) {
        let (tx_call, rx_call) = oneshot::channel();
        (
            Box::new(Self {
                msg,
                tx_call: Some(tx_call),
            }),
            rx_call,
        )
    }
    pub fn new_without_reply(msg: M) -> Box<Self> {
        Box::new(Self { msg, tx_call: None })
    }
}
#[async_trait]
impl<M: Message, H: Handle<M>> ProcessableMessage<H> for Envelope<M, H::Reply> {
    async fn process(self: Box<Self>, actor: &mut H) -> Result<(), H::ActorError> {
        let response = actor
            .handle_fallibly(self.msg)
            .await
            .map_err(|e| e.into())?;
        if let Some(tx_call) = self.tx_call {
            let _ = tx_call
                .send(response)
                .map_err(|_| EnvelopeProcessError::FailedToSend);
        }
        Ok(())
    }
}

macro_rules! impl_message_for_types {
    ($($t:ty),* $(,)?) => {
        $(
            impl crate::Message for $t { }
        )*
    };
}

impl_message_for_types!(
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, String, f32, f64, Box<dyn Message>,
);

#[macro_export]
macro_rules! cast {
    ($actor: expr, $msg: expr) => {
        $actor.cast($msg);
    };
    ($actor: expr, $msg: expr, $timeout: expr) => {
        $actor.cast_timeout($msg, $timeout);
    };
}
#[macro_export]
macro_rules! wait_cast {
    ($actor: expr, $msg: expr) => {
        $actor.wait_cast($msg);
    };
    ($actor: expr, $msg: expr, $timeout: expr) => {
        $actor.cast_timeout($msg, $timeout);
    };
}
#[macro_export]
macro_rules! call {
    ($actor: expr, $msg: expr) => {
        $actor.call($msg)
    };
    ($actor: expr, $msg: expr, $timeout: expr) => {
        $actor.call($msg)
    };
}

pub trait InlineActorHandler<S: InlineActorState> =
    Fn(&mut S, Box<dyn Message>) -> Pin<Box<dyn Future<Output = Box<dyn Message>> + Send>> + Send;
pub trait InlineActorState = 'static + Send;
pub struct InlineActor<S: InlineActorState, H: InlineActorHandler<S> + 'static> {
    handle: H,
    state: S,
}
impl<S: InlineActorState, H: InlineActorHandler<S>> std::fmt::Debug for InlineActor<S, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineActor").finish()
    }
}
impl<S: InlineActorState, H: InlineActorHandler<S>> ActorError for InlineActor<S, H> {}
impl<S: InlineActorState, H: InlineActorHandler<S>> ActorMarker for InlineActor<S, H> {}
impl<S: InlineActorState, H: InlineActorHandler<S>> Actor for InlineActor<S, H> {}

impl<S: InlineActorState, H: InlineActorHandler<S>> InlineActor<S, H> {
    pub fn new(handle: H, state: S) -> Self {
        Self { handle, state }
    }
}

#[async_trait]
impl<M: Message, S: InlineActorState, H: InlineActorHandler<S>> Handle<M> for InlineActor<S, H> {
    type Reply = Box<dyn Message>;
    async fn handle_infallibly(&mut self, msg: M) -> Self::Reply {
        let msg = (self.handle)(&mut self.state, Box::new(msg)).await;
        msg
    }
}
