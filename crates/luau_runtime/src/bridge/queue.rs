use super::handle::HandleMap;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Every mutation a Luau script can request on the Bevy world.
pub enum LuaCommand {
    SpawnPart {
        handle: u64,
        position: Vec3,
        size: Vec3,
        color: Color,
    },
    SetPosition {
        handle: u64,
        value: Vec3,
    },
    SetSize {
        handle: u64,
        value: Vec3,
    },
    SetColor {
        handle: u64,
        r: f32,
        g: f32,
        b: f32,
    },
    Despawn {
        handle: u64,
    },
}

/// Shared command queue — Luau threads push, Bevy drains every frame.
#[derive(Resource, Clone)]
pub struct LuaQueue(pub Arc<Mutex<Vec<LuaCommand>>>);

impl LuaQueue {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }

    pub fn push(&self, cmd: LuaCommand) {
        self.0.lock().unwrap().push(cmd);
    }
}

impl Default for LuaQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy exclusive system — drains the queue and applies every command.
pub fn process_lua_queue(world: &mut World) {
    let queue = world.resource::<LuaQueue>().0.clone();
    let commands: Vec<LuaCommand> = queue.lock().unwrap().drain(..).collect();

    for cmd in commands {
        match cmd {
            LuaCommand::SpawnPart {
                handle,
                position,
                size,
                color,
            } => {
                let mat = world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .add(StandardMaterial::from_color(color));
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(Cuboid::new(size.x, size.y, size.z));

                let entity = world
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(position),
                    ))
                    .id();

                world
                    .resource_mut::<HandleMap>()
                    .insert(handle, entity, Some(mat));
            }

            LuaCommand::SetPosition { handle, value } => {
                if let Some(entity) = world.resource::<HandleMap>().get_entity(handle) {
                    if let Some(mut t) = world.get_mut::<Transform>(entity) {
                        t.translation = value;
                    }
                }
            }

            LuaCommand::SetSize { handle, value } => {
                if let Some(entity) = world.resource::<HandleMap>().get_entity(handle) {
                    if let Some(mut t) = world.get_mut::<Transform>(entity) {
                        t.scale = value;
                    }
                }
            }

            LuaCommand::SetColor { handle, r, g, b } => {
                if let Some(mat_handle) = world.resource::<HandleMap>().get_material(handle) {
                    if let Some(mat) = world
                        .resource_mut::<Assets<StandardMaterial>>()
                        .get_mut(&mat_handle)
                    {
                        mat.base_color = Color::srgb(r, g, b);
                    }
                }
            }

            LuaCommand::Despawn { handle } => {
                if let Some(entry) = world.resource_mut::<HandleMap>().remove(handle) {
                    world.despawn(entry.entity);
                }
            }
        }
    }
}
