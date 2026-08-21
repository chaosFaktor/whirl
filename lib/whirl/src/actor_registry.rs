use std::collections::HashMap;

use crate::{ActorAddr, ActorError, ActorMarker, ErrorHandler};

pub struct ActorRegistry {
    // actors: HashMap<String, Box<ActorAddr<dyn ActorMarker, dyn ActorError<ActorError = dyn std::error::Error, ErrorHandler = dyn ErrorHandler<>>>>>,
}
// impl ActorRegistry {
//     pub fn new() -> Self {
//         Self {
//             actors: HashMap::new(),
//         }
//     }
//     pub fn register(&mut self, key: impl ToString, actor: Box<ActorAddr<dyn ActorMarker>>) {
//         self.actors.insert(key.to_string(), actor);
//     }
//     pub fn deregister(&mut self, key: impl ToString) -> Option<Box<ActorAddr<dyn ActorMarker>>> {
//         return self.actors.remove(&key.to_string());
//     }
//     pub fn get(&mut self, key: impl ToString) -> Option<&mut Box<ActorAddr<dyn ActorMarker>>> {
//         return self.actors.get_mut(&key.to_string());
//     }
// }
