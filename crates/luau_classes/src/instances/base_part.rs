use crate::types::{
    cframe::LuaCFrame, color3::LuaColor3, instance::InstanceData, signal::LuaSignal,
    vector3::LuaVector3,
};
use bevy::{ecs::world::World, math::Vec3, prelude::*, transform::components::Transform};
use bevy_rapier3d::pipeline::CollisionEvent;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle},
        queue::EngineQueue,
    },
    vm::LuaVm,
};

#[derive(Clone)]
pub struct BasePartData {
    pub base: InstanceData,
    pub touched_signal_id: u64,
    pub cframe: LuaCFrame,
    pub size: LuaVector3,
    pub color: LuaColor3,
    pub transparency: f32,
}

pub fn process_collisions(
    mut rapier_msg: MessageReader<CollisionEvent>,
    vm: NonSend<LuaVm>,
    handle_query: Query<&LuauHandle>,
) {
    let Ok(cache) = vm
        .lua
        .named_registry_value::<mlua::Table>("__instance_cache")
    else {
        return;
    };

    for msg in rapier_msg.read() {
        let CollisionEvent::Started(e1, e2, _) = msg else {
            continue;
        };

        for (self_e, other_e) in [(*e1, *e2), (*e2, *e1)] {
            let Ok(handle_self) = handle_query.get(self_e) else {
                continue;
            };
            let Ok(handle_other) = handle_query.get(other_e) else {
                continue;
            };

            let Ok(self_ud) = cache.get::<mlua::AnyUserData>(handle_self.0) else {
                continue;
            };
            let Ok(other_ud) = cache.get::<mlua::AnyUserData>(handle_other.0) else {
                continue;
            };

            if let Ok(part) = self_ud.borrow::<crate::instances::part::LuaPart>() {
                let signal = LuaSignal {
                    id: part.0.touched_signal_id,
                };
                let _ = signal.fire(&vm.lua, other_ud);
            }
        }
    }
}

impl BasePartData {
    pub fn new(handle: u64, queue: EngineQueue, touched_signal_id: u64) -> Self {
        Self {
            base: InstanceData::new(handle, queue, "Part"),
            touched_signal_id,
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
        let h = self.base.handle;
        self.base
            .queue
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
        let h = self.base.handle;
        self.base
            .queue
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
        let h = self.base.handle;
        self.base
            .queue
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
        let h = self.base.handle;
        let t = self.transparency;
        self.base
            .queue
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
        let h = self.base.handle;
        let c = self.color;
        self.base
            .queue
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
        let h = self.base.handle;
        self.base
            .queue
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
