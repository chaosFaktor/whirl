use std::{ io, num::NonZero, time::Duration};

use async_trait::async_trait;
use whirl::{Actor, ActorError, ActorMarker, ErrorHandler, Handle, InlineActorState, Message, PanickingErrorHandler, call, cast, wait_cast};
use whirl_macros::{Actor, Message, actor};
use tokio::{self, pin, time::sleep};


#[derive(Message, Clone, Debug)]
struct MyMsg {}
#[derive(Message)]
struct MyOtherMessage {}

// actor! {
//     pub act A;
//     act MyOtherActor;
//
//     A handle MyMsg (&mut self, msg) -> MyOtherMessage {
//         let a = 12;
//         MyOtherMessage {}
//     }
// }
// #[tokio::test]
// async fn actor_derive_test() {
//     let a = A {};
//     let addr = a.start(1000);
//     let res = call!(addr, MyMsg {}).await.unwrap();
//     assert_eq!(res, MyOtherMessage {});
// }


// #[tokio::test]
// async fn actor_inline_test() {
//     let a = whirl::InlineActor::new(move |state, msg| {
//         let future = async move {
//             Box::new(InlineMessageWrapper(Box::new(12))) as Box<dyn Message>
//         };
//         Box::pin(future)
//     }, 12) ;
//     let handle = a.start(1000);
//     let a = handle.call(12).await.unwrap();
//     let msg = a;
//
//
// }


#[derive(Debug)]
struct MyActor;
impl ActorMarker for MyActor {}
impl Actor for MyActor { }
impl ActorError for MyActor {
    type ErrorHandler = MyErrorHandler;

    type ActorError = io::Error;
}
struct MyErrorHandler;
impl<E: std::error::Error> ErrorHandler<E> for MyErrorHandler {
    fn process_error(error: E) {
        eprintln!("{:?}", error);
        panic!();
    }
    fn install_panic_handler() { }
}

#[async_trait]
impl Handle<MyMsg> for MyActor {
    type Reply = String;
    async fn handle_fallibly(&mut self, _msg: MyMsg) -> Result<Self::Reply, Self::HandleError> {
        // return Err(io::Error::new(io::ErrorKind::Other, anyhow::anyhow!("hi")));
        Ok("hi".to_string())
    }
}



#[tokio::test]
async fn message_test() {
    let msg = MyMsg {};
    let actor = MyActor {};
    let handle = actor.start(100);
    println!("running_state: {:?}", handle.running_state().await);
    handle.cast(MyMsg {}).unwrap();
    handle.cast(msg.clone()).unwrap();
    // sleep(Duration::from_millis(2000)).await;
    // let response = handle.call(msg).await.unwrap();
    // assert_eq!(response, "hi".to_string());
    println!("running_state: {:?}", handle.running_state().await);

    cast!(handle, MyMsg {}).unwrap();
    wait_cast!(handle, MyMsg{}, Duration::from_millis(200)).await.unwrap();
    call!(handle, MyMsg {}).await.unwrap();

}
