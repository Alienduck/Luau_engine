use crate::types::{
    cframe::LuaCFrame, color3::LuaColor3, instance::InstanceData, signal::LuaSignal,
    vector3::LuaVector3,
};
use bevy::{
    ecs::{
        entity::Entity,
        hierarchy::ChildOf,
        message::MessageReader,
        relationship::Relationship,
        system::{NonSend, Query},
        world::World,
    },
    math::Vec3,
};
use luau_runtime::bridge::queue::{EngineCommand, EngineQueue};

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

    pub fn apply_base_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(TouchedSignalComponent(self.touched_signal_id));
        }
    }

    pub fn set_material(&mut self, m: String) {
        self.material = m.clone();
        let (r, g, b) = (self.color.r, self.color.g, self.color.b);
        if m == "Neon" {
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
        if self.material == "Neon" {
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
}

/// Reads Rapier [`CollisionEvent::Started`] messages and fires the Luau
/// `Touched` signal on both involved parts.
pub fn process_collisions(
    mut rapier_msg: MessageReader<bevy_rapier3d::pipeline::CollisionEvent>,
    vm: NonSend<luau_runtime::vm::LuaVm>,
    handle_query: Query<(
        &luau_runtime::bridge::handle::LuauHandle,
        Option<&TouchedSignalComponent>,
    )>,
    parent_query: Query<&ChildOf>,
) {
    let Ok(cache) = vm
        .lua
        .named_registry_value::<mlua::Table>("__instance_cache")
    else {
        return;
    };

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
            let Some((_handle_self, signal_id)) = get_instance_data(*self_e) else {
                continue;
            };
            let Some((handle_other, _)) = get_instance_data(*other_e) else {
                continue;
            };

            if let Some(id) = signal_id {
                if let Ok(other_ud) = cache.get::<mlua::AnyUserData>(handle_other) {
                    let signal = LuaSignal { id };
                    if let Err(e) = signal.fire(&vm.lua, other_ud) {
                        log::error!("Luau Error in Touched event: {}", e);
                    }
                }
            }
        }
    }
}
