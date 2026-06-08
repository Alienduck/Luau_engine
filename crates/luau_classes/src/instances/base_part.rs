use crate::types::{cframe::LuaCFrame, color3::LuaColor3, signal::LuaSignal, vector3::LuaVector3};
use bevy::{ecs::world::World, math::Vec3, prelude::*, transform::components::Transform};
use bevy_rapier3d::pipeline::CollisionEvent;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle},
        queue::EngineQueue,
    },
    vm::LuaVm,
};

pub struct BasePartData {
    pub handle: u64,
    pub touched_signal_id: u64,
    pub queue: EngineQueue,
    pub cframe: LuaCFrame,
    pub size: LuaVector3,
    pub color: LuaColor3,
    pub transparency: f32,
}

#[derive(Message)]
pub struct BasePartTouchedMessage {
    pub entity: Entity,
    pub other_entity: Entity,
}

pub fn process_touched_msg(
    mut messages: MessageReader<BasePartTouchedMessage>,
    vm: NonSend<LuaVm>,
    handle_query: Query<&LuauHandle>,
) {
    let Ok(cache) = vm
        .lua
        .named_registry_value::<mlua::Table>("__instance_cache")
    else {
        return;
    };

    // Just find this, Rust is GOATED
    'test: for msg in messages.read() {
        let Ok(handle_self) = handle_query.get(msg.entity) else {
            continue 'test; // Can name the f*cking core loop (https://doc.rust-lang.org/std/keyword.continue.html)
        };
        let Ok(handle_other) = handle_query.get(msg.other_entity) else {
            continue 'test;
        };
        let Ok(self_instance) = cache.get::<mlua::AnyUserData>(handle_self.0) else {
            continue 'test;
        };
        let Ok(other_instance) = cache.get::<mlua::AnyUserData>(handle_other.0) else {
            continue 'test;
        };
        if let Ok(self_part) = self_instance.borrow::<crate::instances::part::LuaPart>() {
            let signal = LuaSignal {
                id: self_part.0.touched_signal_id,
            };
            let _ = signal.fire(&vm.lua, other_instance);
        }
    }
}

pub fn rapier_collision_bridge(
    mut rapier_events: MessageReader<CollisionEvent>,
    mut touched_messages: MessageWriter<BasePartTouchedMessage>,
) {
    for event in rapier_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            touched_messages.write(BasePartTouchedMessage {
                entity: *e1,
                other_entity: *e2,
            });
            touched_messages.write(BasePartTouchedMessage {
                entity: *e2,
                other_entity: *e1,
            });
        }
    }
}

impl BasePartData {
    pub fn new(handle: u64, queue: EngineQueue, touched_signal_id: u64) -> Self {
        Self {
            handle,
            touched_signal_id,
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
