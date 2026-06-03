use crate::types::{cframe::LuaCFrame, color3::LuaColor3, vector3::LuaVector3};
use bevy::{ecs::world::World, math::Vec3, prelude::*, transform::components::Transform};
use luau_runtime::bridge::{handle::HandleMap, queue::EngineQueue};
use mlua::prelude::*;

/// Data shared by every Part-like instance.
///
/// State (position, size, color) is cached locally so Luau getters work
/// immediately — the queue is write-only and flushed asynchronously by Bevy,
/// so reading it back would never give the right answer.
pub struct BasePartData {
    pub handle: u64,
    pub queue: EngineQueue,
    pub cframe: LuaCFrame,
    pub size: LuaVector3,
    pub color: LuaColor3,
    pub transparency: LuaValue,
}

impl BasePartData {
    pub fn new(handle: u64, queue: EngineQueue) -> Self {
        Self {
            handle,
            queue,
            cframe: LuaCFrame::default(),
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
            transparency: LuaValue::Number(0.0),
        }
    }

    pub fn set_position(&mut self, position: LuaVector3) {
        self.cframe.position = Vec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        };
        let handle = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(handle) {
                    if let Some(mut t) = w.get_mut::<Transform>(e) {
                        t.translation = Vec3 {
                            x: position.x,
                            y: position.y,
                            z: position.z,
                        }
                    }
                }
            }));
    }

    pub fn set_cframe(&mut self, cf: LuaCFrame) {
        self.cframe = cf;
        let handle = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(handle) {
                    if let Some(mut t) = w.get_mut::<Transform>(e) {
                        t.translation = cf.position;
                        t.rotation = cf.rotation;
                    }
                }
            }));
    }

    pub fn set_size(&mut self, v: LuaVector3) {
        self.size = v;
        let handle = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(handle) {
                    if let Some(mut t) = w.get_mut::<Transform>(e) {
                        t.scale = Vec3 {
                            x: v.x,
                            y: v.y,
                            z: v.z,
                        }
                    }
                }
            }));
    }

    pub fn set_color(&mut self, c: LuaColor3) {
        self.color = c;
        let target_handle = self.handle;

        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(target_handle) {
                    let cloned_handle = w
                        .get::<MeshMaterial3d<StandardMaterial>>(e)
                        .map(|mat| mat.0.clone());

                    if let Some(mat_handle) = cloned_handle {
                        if let Some(mat) = w
                            .resource_mut::<Assets<StandardMaterial>>()
                            .get_mut(&mat_handle)
                        {
                            mat.base_color = Color::srgb(c.r, c.g, c.b);
                        }
                    }
                }
            }));
    }

    pub fn set_transparency(&mut self, v: LuaValue) {
        let Some(t) = v.as_f32() else { return };
        self.transparency = v;
        let target_handle = self.handle;

        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(target_handle) {
                    let cloned_handle = w
                        .get::<MeshMaterial3d<StandardMaterial>>(e)
                        .map(|mat| mat.0.clone());

                    if let Some(mat_handle) = cloned_handle {
                        if let Some(mat) = w
                            .resource_mut::<Assets<StandardMaterial>>()
                            .get_mut(&mat_handle)
                        {
                            mat.base_color.set_alpha(1.0 - t);
                            if mat.base_color.alpha() < 1.0 {
                                mat.alpha_mode = AlphaMode::Blend;
                            } else {
                                mat.alpha_mode = AlphaMode::Opaque
                            }
                        }
                    }
                }
            }));
    }

    pub fn destroy(&self) {
        let handle = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource_mut::<HandleMap>().remove(handle) {
                    w.despawn(e.entity);
                }
            }));
    }
}
