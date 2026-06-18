use crate::types::{
    cframe::LuaCFrame, color3::LuaColor3, instance::InstanceData, signal::LuaSignal,
    vector3::LuaVector3,
};
use bevy::{
    ecs::{relationship::Relationship, world::World},
    math::{Vec3, VectorSpace},
    prelude::*,
};
use bevy_rapier3d::pipeline::CollisionEvent;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle},
        queue::EngineQueue,
    },
    vm::LuaVm,
};

/// Shared state for 3-D part instances (position, size, color, transparency,
/// touched signal).
///
/// Embedded by value in [`LuaPart`] (and any future part subclass).  All
/// mutations enqueue a corresponding Bevy world command via `base.queue`.
#[derive(Clone)]
pub struct BasePartData {
    pub base: InstanceData,
    /// Signal fired when another part begins overlapping this one.
    pub touched_signal_id: u64,
    /// Position and rotation, kept in sync with the Bevy `Transform`.
    pub cframe: LuaCFrame,
    /// Full size (= Bevy `Transform::scale` for a unit-cube mesh).
    pub size: LuaVector3,
    pub color: LuaColor3,
    /// 0.0 = fully opaque, 1.0 = fully transparent.
    pub transparency: f32,
    pub material: String,
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
            material: "Plastic".into(),
        }
    }

    pub fn set_material(&mut self, m: String) {
        self.material = m.clone();
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
                            mat.emissive = if m == "Neon" {
                                LinearRgba::rgb(c.r * 10.0, c.g * 10.0, c.b * 10.0)
                            } else {
                                LinearRgba::ZERO
                            };
                        }
                    }
                }
            }));
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
        let m = self.material.clone();
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
                            mat.emissive = if m == "Neon" {
                                LinearRgba::rgb(c.r * 10.0, c.g * 10.0, c.b * 10.0)
                            } else {
                                LinearRgba::ZERO
                            };
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
}

/// Reads Rapier [`CollisionEvent::Started`] messages and fires the Luau
/// `Touched` signal on both involved parts.
///
/// Requires `MessageReader` (Bevy 0.17+ buffered event API) rather than the
/// observer-based `EventReader`.
pub fn process_collisions(
    mut rapier_msg: MessageReader<CollisionEvent>,
    vm: NonSend<LuaVm>,
    handle_query: Query<&LuauHandle>,
    parent_query: Query<&ChildOf>,
) {
    let Ok(cache) = vm
        .lua
        .named_registry_value::<mlua::Table>("__instance_cache")
    else {
        return;
    };

    let get_luau_handle = |mut entity: Entity| -> Option<u64> {
        loop {
            if let Ok(handle) = handle_query.get(entity) {
                return Some(handle.0);
            }
            if let Ok(parent) = parent_query.get(entity) {
                entity = parent.get();
            } else {
                return None;
            }
        }
    };

    for msg in rapier_msg.read() {
        let CollisionEvent::Started(e1, e2, _) = msg else {
            continue;
        };

        for (self_e, other_e) in [(*e1, *e2), (*e2, *e1)] {
            let Some(handle_self) = get_luau_handle(self_e) else {
                continue;
            };
            let Some(handle_other) = get_luau_handle(other_e) else {
                continue;
            };

            let Ok(self_ud) = cache.get::<mlua::AnyUserData>(handle_self) else {
                continue;
            };
            let Ok(other_ud) = cache.get::<mlua::AnyUserData>(handle_other) else {
                continue;
            };

            let signal_id = if let Ok(part) = self_ud.borrow::<crate::instances::part::LuaPart>() {
                Some(part.data.touched_signal_id)
            } else if let Ok(mpart) = self_ud.borrow::<crate::instances::mesh_part::LuaMeshPart>() {
                Some(mpart.base_part_data.touched_signal_id)
            } else {
                None
            };

            if let Some(id) = signal_id {
                let signal = LuaSignal { id };
                if let Err(e) = signal.fire(&vm.lua, other_ud) {
                    log::error!("Luau Error in Touched event: {}", e);
                }
            }
        }
    }
}
