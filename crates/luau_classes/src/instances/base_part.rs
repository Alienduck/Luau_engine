use crate::types::{color3::LuaColor3, vector3::LuaVector3};
use luau_runtime::bridge::queue::{LuaCommand, LuaQueue};
use std::sync::{Arc, Mutex};

/// Data shared by every Part-like instance.
pub struct BasePartData {
    pub handle: u64,
    pub queue: Arc<Mutex<Vec<LuaCommand>>>,
}

impl BasePartData {
    pub fn new(handle: u64, queue: Arc<Mutex<Vec<LuaCommand>>>) -> Self {
        Self { handle, queue }
    }

    pub fn get_position(&self) -> LuaVector3 {
        LuaVector3 {
            x: self
                .queue
                .lock()
                .unwrap()
                .iter()
                .find(|c| matches!(c, LuaCommand::SetPosition { .. }))
                .map_or(0.0, |c| match c {
                    LuaCommand::SetPosition { value, .. } => value.x,
                    _ => 0.0,
                }),
            y: self
                .queue
                .lock()
                .unwrap()
                .iter()
                .find(|c| matches!(c, LuaCommand::SetPosition { .. }))
                .map_or(0.0, |c| match c {
                    LuaCommand::SetPosition { value, .. } => value.y,
                    _ => 0.0,
                }),
            z: self
                .queue
                .lock()
                .unwrap()
                .iter()
                .find(|c| matches!(c, LuaCommand::SetPosition { .. }))
                .map_or(0.0, |c| match c {
                    LuaCommand::SetPosition { value, .. } => value.z,
                    _ => 0.0,
                }),
        }
    }

    pub fn set_position(&self, v: LuaVector3) {
        self.queue.lock().unwrap().push(LuaCommand::SetPosition {
            handle: self.handle,
            value: bevy::math::Vec3::new(v.x, v.y, v.z),
        });
    }

    pub fn set_size(&self, v: LuaVector3) {
        self.queue.lock().unwrap().push(LuaCommand::SetSize {
            handle: self.handle,
            value: bevy::math::Vec3::new(v.x, v.y, v.z),
        });
    }

    pub fn set_color(&self, c: LuaColor3) {
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
