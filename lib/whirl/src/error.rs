use std::marker::PhantomData;

use tokio::{sync::{mpsc::error::{SendError, SendTimeoutError}, oneshot}, time::error::Elapsed};

use crate::{Actor, ActorError, Handle, Message, ProcessableMessage};


#[derive(thiserror::Error, Debug)]
pub enum CallError<M: Message, H: Handle<M>> {
    #[error("Error reading repsonse from actor")]
    RecvError(#[from] oneshot::error::RecvError),
    #[error("Error sending message to actor")]
    SendError(#[from] SendError<Box<dyn ProcessableMessage<H>>>),

    #[error("phantomdata")]
    _Marker(PhantomData<M>),
}
#[derive(thiserror::Error, Debug)]
pub enum CallTimeoutError<M: Message, H: Handle<M>> {
    #[error("Error reading repsonse from actor")]
    RecvError(#[from] oneshot::error::RecvError),
    #[error("Send Timeout error")]
    SendTimoutError(#[from] SendTimeoutError<Box<dyn ProcessableMessage<H>>>),
    #[error("Timeout error")]
    TimeoutError(#[from] Elapsed),

    #[error("phantomdata")]
    _Marker(PhantomData<M>),
}

#[derive(thiserror::Error, Debug)]
pub enum EnvelopeProcessError {
    #[error("failed to send")]
    FailedToSend,
}


#[derive(thiserror::Error, Debug)]
pub enum AnyError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Boxed(#[from] Box<dyn std::error::Error>)
}

