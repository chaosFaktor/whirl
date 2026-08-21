// use async_trait::async_trait;
//
// use crate::{Actor, ActorMessage, Envelope, Handle, Message};
//
// pub struct HelloWorldActor {}
// impl Actor for HelloWorldActor {}
//
// pub struct StringWrapper(String);
// impl Message for StringWrapper {}
//
// #[async_trait]
// impl Handle<StringWrapper> for HelloWorldActor {
//     async fn handle(&mut self, msg: &StringWrapper) {
//         println!("Hello {}", msg.0)
//     }
// }
//
// #[tokio::test]
// pub async fn hello_world_test() {
//     let myactor = HelloWorldActor {};
//     let addr = myactor.start();
//     addr.send(Envelope::new(StringWrapper("hi".to_string())))
//         .unwrap();
// }
