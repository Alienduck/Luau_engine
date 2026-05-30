use crate::types::{color3::LuaColor3, vector3::LuaVector3};
use luau_runtime::bridge::queue::LuaCommand;
use std::sync::{Arc, Mutex};

/// Data shared by every Part-like instance.
///
/// State (position, size, color) is cached locally so Luau getters work
/// immediately — the queue is write-only and flushed asynchronously by Bevy,
/// so reading it back would never give the right answer.
pub struct BasePartData {
    pub handle: u64,
    pub queue: Arc<Mutex<Vec<LuaCommand>>>,
    // Local cache — updated on every set, readable at any time from Luau.
    pub position: LuaVector3,
    pub size: LuaVector3,
    pub color: LuaColor3,
}

impl BasePartData {
    pub fn new(handle: u64, queue: Arc<Mutex<Vec<LuaCommand>>>) -> Self {
        Self {
            handle,
            queue,
            position: LuaVector3::default(),
            size: LuaVector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            color: LuaColor3 {
                r: 0.8,
                g: 0.8,
                b: 0.8,
            },
        }
    }

    pub fn set_position(&mut self, v: LuaVector3) {
        self.position = v;
        self.queue.lock().unwrap().push(LuaCommand::SetPosition {
            handle: self.handle,
            value: bevy::math::Vec3::new(v.x, v.y, v.z),
        });
    }

    pub fn set_size(&mut self, v: LuaVector3) {
        self.size = v;
        self.queue.lock().unwrap().push(LuaCommand::SetSize {
            handle: self.handle,
            value: bevy::math::Vec3::new(v.x, v.y, v.z),
        });
    }

    pub fn set_color(&mut self, c: LuaColor3) {
        self.color = c;
        self.queue.lock().unwrap().push(LuaCommand::SetColor {
            handle: self.handle,
            r: c.r,
            g: c.g,
            b: c.b,
        });
    }

    pub fn destroy(&self) {
        self.queue.lock().unwrap().push(LuaCommand::Despawn {
            handle: self.handle,
        });
    }
}
