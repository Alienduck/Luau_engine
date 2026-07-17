use crate::types::{
    cframe::LuaCFrame, color3::LuaColor3, enums::BasePartMaterial, instance::InstanceData,
    signal::LuaSignal, vector3::LuaVector3,
};
use bevy::{
    ecs::{
        entity::Entity,
        hierarchy::ChildOf,
        message::MessageReader,
        query::Changed,
        relationship::Relationship,
        resource::Resource,
        system::{NonSend, Query, ResMut},
        world::World,
    },
    math::Vec3,
    transform::components::Transform,
};
use luau_runtime::{
    bridge::{
        handle::LuauHandle,
        queue::{EngineCommand, EngineQueue},
    },
    vm::LuaVm,
};

#[derive(bevy::ecs::component::Component)]
pub struct TouchedSignalComponent(pub u64);

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
    pub material: BasePartMaterial,
    pub shadow_caster: bool,
    pub shadow_receiver: bool,
}

impl BasePartData {
    pub fn new(
        handle: u64,
        queue: EngineQueue,
        touched_signal_id: u64,
        destroying_signal_id: u64,
    ) -> Self {
        Self {
            base: InstanceData::new(handle, queue, "Part", destroying_signal_id),
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
            material: BasePartMaterial::Plastic,
            shadow_caster: true,
            shadow_receiver: true,
        }
    }

    pub fn apply_base_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(TouchedSignalComponent(self.touched_signal_id));
        }
    }

    pub fn set_material(&mut self, m: BasePartMaterial) {
        self.material = m;
        let (r, g, b) = (self.color.r, self.color.g, self.color.b);
        if m == BasePartMaterial::Neon {
            self.base.queue.push(EngineCommand::SetEmissive {
                handle: self.base.handle,
                r: r * 10.0,
                g: g * 10.0,
                b: b * 10.0,
            });
        } else {
            self.base.queue.push(EngineCommand::SetEmissive {
                handle: self.base.handle,
                r: 0.0,
                g: 0.0,
                b: 0.0,
            });
        }
    }

    pub fn set_position(&mut self, p: LuaVector3) {
        self.cframe.position = Vec3::new(p.x, p.y, p.z);
        self.base.queue.push(EngineCommand::SetTranslation {
            handle: self.base.handle,
            translation: Vec3::new(p.x, p.y, p.z),
        });
    }

    pub fn set_orientation(&mut self, o: LuaVector3) {
        self.cframe.rotation = o.into();
        self.base.queue.push(EngineCommand::SetRotation {
            handle: self.base.handle,
            rotation: o.into(),
        });
    }

    pub fn set_cframe(&mut self, cf: LuaCFrame) {
        self.cframe = cf;
        self.base.queue.push(EngineCommand::SetCFrame {
            handle: self.base.handle,
            translation: cf.position,
            rotation: cf.rotation,
        });
    }

    pub fn set_size(&mut self, v: LuaVector3) {
        self.size = v;
        self.base.queue.push(EngineCommand::SetScale {
            handle: self.base.handle,
            scale: Vec3::new(v.x, v.y, v.z),
        });
    }

    pub fn set_color(&mut self, c: LuaColor3) {
        self.color = c;
        let alpha = 1.0 - self.transparency;
        self.base.queue.push(EngineCommand::SetBaseColor {
            handle: self.base.handle,
            r: c.r,
            g: c.g,
            b: c.b,
            alpha,
        });
        if self.material == BasePartMaterial::Neon {
            self.base.queue.push(EngineCommand::SetEmissive {
                handle: self.base.handle,
                r: c.r * 10.0,
                g: c.g * 10.0,
                b: c.b * 10.0,
            });
        }
    }

    pub fn set_transparency(&mut self, t: f32) {
        self.transparency = t;
        self.base.queue.push(EngineCommand::SetBaseColor {
            handle: self.base.handle,
            r: self.color.r,
            g: self.color.g,
            b: self.color.b,
            alpha: 1.0 - t,
        });
    }

    pub fn set_shadow_cast(&mut self, s: bool) {
        self.shadow_caster = s;
        self.base.queue.push(EngineCommand::SetShadowCast {
            handle: self.base.handle,
            cast: s,
        });
    }

    pub fn set_shadow_receiver(&mut self, r: bool) {
        self.shadow_receiver = r;
        self.base.queue.push(EngineCommand::SetShadowReceiver {
            handle: self.base.handle,
            receive: r,
        });
    }
}

#[derive(Resource, Default)]
pub struct PendingTouches(pub Vec<(u64, u64)>);

/// Reads Rapier [`CollisionEvent::Started`] messages and place it in `PendingTouches`
/// `Touched` signal on both involved parts.
pub fn process_collisions(
    mut rapier_msg: MessageReader<bevy_rapier3d::pipeline::CollisionEvent>,
    handle_query: Query<(
        &luau_runtime::bridge::handle::LuauHandle,
        Option<&TouchedSignalComponent>,
    )>,
    parent_query: Query<&ChildOf>,
    mut pending: ResMut<PendingTouches>,
) {
    let get_instance_data = |mut entity: bevy::prelude::Entity| -> Option<(u64, Option<u64>)> {
        loop {
            if let Ok((handle, signal_comp)) = handle_query.get(entity) {
                return Some((handle.0, signal_comp.map(|s| s.0)));
            }
            if let Ok(parent) = parent_query.get(entity) {
                entity = parent.get();
            } else {
                return None;
            }
        }
    };
    for msg in rapier_msg.read() {
        let bevy_rapier3d::pipeline::CollisionEvent::Started(e1, e2, _) = msg else {
            continue;
        };
        for (self_e, other_e) in [(e1, e2), (e2, e1)] {
            let Some((_, signal_id)) = get_instance_data(*self_e) else {
                continue;
            };
            let Some((handle_other, _)) = get_instance_data(*other_e) else {
                continue;
            };
            if let Some(id) = signal_id {
                pending.0.push((id, handle_other));
            }
        }
    }
}

/// Process the collision signals
pub fn flush_touched_signals(
    vm: NonSend<luau_runtime::vm::LuaVm>,
    mut pending: ResMut<PendingTouches>,
) {
    let Ok(cache) = vm
        .lua
        .named_registry_value::<mlua::Table>("__instance_cache")
    else {
        return;
    };
    for (signal_id, other_handle) in pending.0.drain(..) {
        if let Ok(other_ud) = cache.get::<mlua::AnyUserData>(other_handle) {
            let signal = LuaSignal { id: signal_id };
            let _ = signal.fire(&vm.lua, other_ud);
        }
    }
}

/// Update LuaPosition of BasePart updated by Rapier's physic
pub fn sync_transforms_system(
    vm: NonSend<LuaVm>,
    query: Query<(&LuauHandle, &Transform), Changed<Transform>>,
) {
    let Ok(cache) = vm
        .lua
        .named_registry_value::<mlua::Table>("__instance_cache")
    else {
        return;
    };

    for (handle, transform) in query.iter() {
        if let Ok(ud) = cache.get::<mlua::AnyUserData>(handle.0) {
            if let Ok(mut part) = ud.borrow_mut::<crate::instances::part::LuaPart>() {
                part.data.cframe.position = transform.translation;
                part.data.cframe.rotation = transform.rotation;
            } else if let Ok(mut mesh_part) =
                ud.borrow_mut::<crate::instances::mesh_part::LuaMeshPart>()
            {
                mesh_part.base_part_data.cframe.position = transform.translation;
                mesh_part.base_part_data.cframe.rotation = transform.rotation;
            }
        }
    }
}
