use std::str::FromStr;

use super::base_part::BasePartData;
use crate::types::{
    cframe::LuaCFrame,
    color3::LuaColor3,
    enums::{BasePartMaterial, LuauPartShape},
    instance::{CloneableInstance, InstanceData},
    signal::LuaSignal,
    vector3::LuaVector3,
};
use bevy::{
    asset::Assets,
    color::Color,
    ecs::world::World,
    math::{VectorSpace, primitives::Cuboid},
    pbr::StandardMaterial,
    prelude::*,
};
use luau_runtime::bridge::{
    handle::{HandleMap, next_handle},
    queue::EngineQueue,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

/// Luau-facing `Part` instance — a renderable 3-D box with optional physics.
///
/// Wraps [`BasePartData`] which holds the shared 3-D instance state (position,
/// size, color, transparency, touched signal).
#[derive(Clone)]
pub struct LuaPart {
    pub data: BasePartData,
    pub shape: u8,
}

impl CloneableInstance for LuaPart {
    fn base(&self) -> &InstanceData {
        &self.data.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.data.base
    }
    fn on_cloned(&mut self, lua: &mlua::Lua) -> mlua::Result<()> {
        self.data.touched_signal_id = LuaSignal::new(lua)?.id;
        Ok(())
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        self.data.apply_base_components(entity, w);
        let emissive = if self.data.material == BasePartMaterial::Neon {
            LinearRgba::rgb(
                self.data.color.r * 10.0,
                self.data.color.g * 10.0,
                self.data.color.b * 10.0,
            )
        } else {
            LinearRgba::ZERO
        };
        let mat = w
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgba(
                    self.data.color.r,
                    self.data.color.g,
                    self.data.color.b,
                    1.0 - self.data.transparency,
                ),
                emissive,
                alpha_mode: if self.data.transparency > 0.0 {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                ..default()
            });
        let (mesh, shape_comp) = match self.shape {
            1 => (
                w.resource_mut::<Assets<Mesh>>().add(Sphere::new(0.5)),
                LuauPartShape::Ball,
            ),
            2 => (
                w.resource_mut::<Assets<Mesh>>()
                    .add(Cylinder::new(0.5, 1.0)),
                LuauPartShape::Cylinder,
            ),
            _ => (
                w.resource_mut::<Assets<Mesh>>().add(Cuboid::default()),
                LuauPartShape::Block,
            ),
        };
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert((Mesh3d(mesh), MeshMaterial3d(mat), shape_comp));
            if let Some(mut t) = e.get_mut::<Transform>() {
                t.translation = self.data.cframe.position;
                t.rotation = self.data.cframe.rotation;
                t.scale = Vec3::new(self.data.size.x, self.data.size.y, self.data.size.z);
            }
        }
    }
}

impl UserData for LuaPart {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);
        fields.add_field_method_get("Position", |_, this| {
            Ok(LuaVector3 {
                x: this.data.cframe.position.x,
                y: this.data.cframe.position.y,
                z: this.data.cframe.position.z,
            })
        });
        fields.add_field_method_get("CFrame", |_, this| Ok(this.data.cframe));
        fields.add_field_method_get("Size", |_, this| Ok(this.data.size));
        fields.add_field_method_get("Color", |_, this| Ok(this.data.color));
        fields.add_field_method_get("Transparency", |_, this| Ok(this.data.transparency));
        fields.add_field_method_get("Touched", |_, this| {
            Ok(LuaSignal {
                id: this.data.touched_signal_id,
            })
        });
        fields.add_field_method_get("Material", |_, this| {
            Ok(this.data.material.as_ref().to_string())
        });
        fields.add_field_method_get("Shape", |_, this| Ok(this.shape));
        fields.add_field_method_set("Position", |_, this, v: LuaVector3| {
            this.data.set_position(v);
            Ok(())
        });
        fields.add_field_method_set("CFrame", |_, this, v: LuaCFrame| {
            this.data.set_cframe(v);
            Ok(())
        });
        fields.add_field_method_set("Size", |_, this, v: LuaVector3| {
            this.data.set_size(v);
            Ok(())
        });
        fields.add_field_method_set("Color", |_, this, c: LuaColor3| {
            this.data.set_color(c);
            Ok(())
        });
        fields.add_field_method_set("Transparency", |_, this, t: f32| {
            this.data.set_transparency(t);
            Ok(())
        });
        fields.add_field_method_set("Material", |_, this, v: String| {
            this.data
                .set_material(BasePartMaterial::from_str(&v).unwrap_or_default());
            Ok(())
        });
        fields.add_field_method_set("Shape", |_, this, v: u8| {
            this.shape = v;
            let h = this.data.base.handle;
            this.data.base.queue.push_raw(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    let (mesh, shape_comp) = match v {
                        1 => (
                            w.resource_mut::<Assets<Mesh>>().add(Sphere::new(0.5)),
                            LuauPartShape::Ball,
                        ),
                        2 => (
                            w.resource_mut::<Assets<Mesh>>()
                                .add(Cylinder::new(0.5, 1.0)),
                            LuauPartShape::Cylinder,
                        ),
                        3 => (
                            w.resource_mut::<Assets<Mesh>>()
                                .add(Capsule3d::new(0.5, 1.0)),
                            LuauPartShape::Capsule,
                        ),
                        _ => (
                            w.resource_mut::<Assets<Mesh>>().add(Cuboid::default()),
                            LuauPartShape::Block,
                        ),
                    };
                    if let Ok(mut em) = w.get_entity_mut(e) {
                        em.insert((Mesh3d(mesh), shape_comp));
                    }
                }
            });
            Ok(())
        });
        fields.add_field_method_get("Rotation", |_, this| {
            Ok(LuaVector3::from(this.data.cframe.rotation))
        });
        fields.add_field_method_set("Rotation", |_, this, v: LuaVector3| {
            this.data.set_orientation(v);
            Ok(())
        });
        fields.add_field_method_get("CastShadow", |_, this| Ok(this.data.shadow_caster));
        fields.add_field_method_set("CastShadow", |_, this, v: bool| {
            this.data.set_shadow_cast(v);
            Ok(())
        });
        fields.add_field_method_get("ReceiveShadow", |_, this| Ok(this.data.shadow_receiver));
        fields.add_field_method_set("ReceiveShadow", |_, this, v: bool| {
            this.data.set_shadow_receiver(v);
            Ok(())
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.data.base.name.clone()));
    }
}

pub struct PartModule;

impl luau_runtime::registry::LuaModule for PartModule {
    fn name() -> &'static str {
        "Part"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let touched_signal = LuaSignal::new(lua_ctx)?;
                let destroying_signal_id = crate::types::signal::LuaSignal::new(lua_ctx)?.id;
                let part = LuaPart {
                    data: BasePartData::new(
                        handle,
                        q.clone(),
                        touched_signal.id,
                        destroying_signal_id,
                    ),
                    shape: 0,
                };
                let spawn_copy = part.clone();
                q.push_raw(move |w: &mut World| {
                    let entity = spawn_copy.base().spawn_base_entity(w);
                    spawn_copy.apply_bevy_components(entity, w);
                });
                let ud = lua_ctx.create_userdata(part)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("Part", t)
    }
}
