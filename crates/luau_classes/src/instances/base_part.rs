use crate::types::{cframe::LuaCFrame, color3::LuaColor3, vector3::LuaVector3};
use bevy::{ecs::world::World, math::Vec3, prelude::*, transform::components::Transform};
use luau_runtime::bridge::{handle::HandleMap, queue::EngineQueue};

pub struct BasePartData {
    pub handle: u64,
    pub queue: EngineQueue,
    pub cframe: LuaCFrame,
    pub size: LuaVector3,
    pub color: LuaColor3,
    pub transparency: f32,
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
            transparency: 0.0,
        }
    }

    pub fn set_position(&mut self, p: LuaVector3) {
        self.cframe.position = Vec3::new(p.x, p.y, p.z);
        let h = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    if let Some(mut t) = w.get_mut::<Transform>(e) {
                        t.translation = Vec3::new(p.x, p.y, p.z);
                    }
                }
            }));
    }

    pub fn set_cframe(&mut self, cf: LuaCFrame) {
        self.cframe = cf;
        let h = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    if let Some(mut t) = w.get_mut::<Transform>(e) {
                        t.translation = cf.position;
                        t.rotation = cf.rotation;
                    }
                }
            }));
    }

    pub fn set_size(&mut self, v: LuaVector3) {
        self.size = v;
        let h = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    if let Some(mut t) = w.get_mut::<Transform>(e) {
                        t.scale = Vec3::new(v.x, v.y, v.z);
                    }
                }
            }));
    }

    pub fn set_color(&mut self, c: LuaColor3) {
        self.color = c;
        let h = self.handle;
        let t = self.transparency;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    if let Some(mat_h) = w
                        .get::<MeshMaterial3d<StandardMaterial>>(e)
                        .map(|m| m.0.clone())
                    {
                        if let Some(mat) =
                            w.resource_mut::<Assets<StandardMaterial>>().get_mut(&mat_h)
                        {
                            mat.base_color = Color::srgba(c.r, c.g, c.b, 1.0 - t);
                        }
                    }
                }
            }));
    }

    pub fn set_transparency(&mut self, t: f32) {
        self.transparency = t;
        let h = self.handle;
        let c = self.color;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    if let Some(mat_h) = w
                        .get::<MeshMaterial3d<StandardMaterial>>(e)
                        .map(|m| m.0.clone())
                    {
                        if let Some(mat) =
                            w.resource_mut::<Assets<StandardMaterial>>().get_mut(&mat_h)
                        {
                            mat.base_color = Color::srgba(c.r, c.g, c.b, 1.0 - t);
                            mat.alpha_mode = if t > 0.0 {
                                AlphaMode::Blend
                            } else {
                                AlphaMode::Opaque
                            };
                        }
                    }
                }
            }));
    }

    pub fn destroy(&self) {
        let h = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource_mut::<HandleMap>().remove(h) {
                    w.despawn(e.entity);
                }
            }));
    }
}
