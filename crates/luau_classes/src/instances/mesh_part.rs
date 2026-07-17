use crate::{
    impl_base_instance_fields,
    instances::base_part::BasePartData,
    types::{
        instance::{CloneableInstance, InstanceData},
        signal::LuaSignal,
        vector3::LuaVector3,
    },
};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

#[derive(Component, Clone, Copy, PartialEq)]
pub enum LuauCollisionFidelity {
    Default,
    Hull,
    Box,
    Precise,
}

impl Default for LuauCollisionFidelity {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone)]
pub struct LuaMeshPart {
    pub base_part_data: BasePartData,
    pub mesh_id: String,
    pub collision_fidelity: u8,
}

impl CloneableInstance for LuaMeshPart {
    fn base(&self) -> &InstanceData {
        &self.base_part_data.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base_part_data.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        let comp = match self.collision_fidelity {
            1 => LuauCollisionFidelity::Hull,
            2 => LuauCollisionFidelity::Box,
            3 => LuauCollisionFidelity::Precise,
            _ => LuauCollisionFidelity::Default,
        };
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(comp);
        }
    }
}

impl UserData for LuaMeshPart {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        impl_base_instance_fields!(fields);
        fields.add_field_method_get("MeshId", |_, this| Ok(this.mesh_id.clone()));
        fields.add_field_method_set("MeshId", |_, this, v: String| {
            this.mesh_id = v.clone();
            let h = this.base_part_data.base.handle;
            this.base_part_data.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::LoadAsset {
                    handle: h,
                    asset_path: v,
                },
            );
            Ok(())
        });
        fields.add_field_method_get("Position", |_, this| {
            let p = this.base_part_data.cframe.position;
            Ok(LuaVector3 {
                x: p.x,
                y: p.y,
                z: p.z,
            })
        });
        fields.add_field_method_set("Position", |_, this, v: LuaVector3| {
            this.base_part_data.set_position(v);
            Ok(())
        });
        fields.add_field_method_get("Size", |_, this| Ok(this.base_part_data.size));
        fields.add_field_method_set("Size", |_, this, v: LuaVector3| {
            this.base_part_data.size = v;
            let h = this.base_part_data.base.handle;
            this.base_part_data.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetScale {
                    handle: h,
                    scale: Vec3 {
                        x: v.x,
                        y: v.y,
                        z: v.z,
                    },
                },
            );
            Ok(())
        });
        fields.add_field_method_get("Transparency", |_, this| {
            Ok(this.base_part_data.transparency)
        });
        fields.add_field_method_set("Transparency", |_, this, t: f32| {
            this.base_part_data.set_transparency(t);
            Ok(())
        });

        fields.add_field_method_get("Color", |_, this| Ok(this.base_part_data.color));
        fields.add_field_method_set("Color", |_, this, c: crate::types::color3::LuaColor3| {
            this.base_part_data.set_color(c);
            Ok(())
        });
        fields.add_field_method_get("CollisionFidelity", |_, this| Ok(this.collision_fidelity));
        fields.add_field_method_set("CollisionFidelity", |_, this, v: u8| {
            this.collision_fidelity = v;
            let h = this.base_part_data.base.handle;
            this.base_part_data
                .base
                .queue
                .push_raw(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        let comp = match v {
                            1 => LuauCollisionFidelity::Hull,
                            2 => LuauCollisionFidelity::Box,
                            3 => LuauCollisionFidelity::Precise,
                            _ => LuauCollisionFidelity::Default,
                        };
                        if let Ok(mut em) = w.get_entity_mut(e) {
                            em.insert(comp);
                        }
                    }
                });
            Ok(())
        });
        fields.add_field_method_get("Rotation", |_, this| {
            Ok(LuaVector3::from(this.base_part_data.cframe.rotation))
        });
        fields.add_field_method_set("Rotation", |_, this, v: LuaVector3| {
            this.base_part_data.set_orientation(v);
            Ok(())
        });
        fields.add_field_method_get("CastShadow", |_, this| {
            Ok(this.base_part_data.shadow_caster)
        });
        fields.add_field_method_set("CastShadow", |_, this, v: bool| {
            this.base_part_data.set_shadow_cast(v);
            Ok(())
        });
        fields.add_field_method_get("ReceiveShadow", |_, this| {
            Ok(this.base_part_data.shadow_receiver)
        });
        fields.add_field_method_set("ReceiveShadow", |_, this, v: bool| {
            this.base_part_data.set_shadow_receiver(v);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base().name.clone()));
    }
}

pub struct MeshPartModule;

impl LuaModule for MeshPartModule {
    fn name() -> &'static str {
        "MeshPart"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let touch_signal_id = LuaSignal::new(lua_ctx)?;
                let destroying_signal_id = crate::types::signal::LuaSignal::new(lua_ctx)?.id;
                let mesh_part = LuaMeshPart {
                    base_part_data: BasePartData::new(
                        handle,
                        q.clone(),
                        touch_signal_id.id,
                        destroying_signal_id,
                    ),
                    mesh_id: "".to_string(),
                    collision_fidelity: 0,
                };
                let clone_for_spawn = mesh_part.clone();
                q.push_raw(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Transform::default(),
                            Visibility::default(),
                            luau_runtime::bridge::handle::LuauHandle(handle),
                        ))
                        .id();
                    clone_for_spawn.apply_bevy_components(entity, w);
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                });
                let ud = lua_ctx.create_userdata(mesh_part)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("MeshPart", t)
    }
}
