use bevy::prelude::*;
use std::sync::{Arc, Mutex};

pub type WorldCommand = Box<dyn FnOnce(&mut World) + Send + Sync>;

#[derive(Resource, Clone, Default)]
pub struct EngineQueue(pub Arc<Mutex<Vec<WorldCommand>>>);

pub fn process_engine_queue(world: &mut World) {
    let queue = world.resource::<EngineQueue>().0.clone();
    for cmd in queue.lock().unwrap().drain(..) {
        cmd(world);
    }
}
