use super::base_part::BasePartData;
use crate::types::{cframe::LuaCFrame, color3::LuaColor3, signal::LuaSignal, vector3::LuaVector3};
use bevy::{
    asset::Assets, color::Color, ecs::world::World, math::primitives::Cuboid,
    pbr::StandardMaterial, prelude::*,
};
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

pub struct LuaPart(pub BasePartData);

impl UserData for LuaPart {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.0.base.name.clone()));
        fields.add_field_method_get("Position", |_, this| {
            Ok(LuaVector3 {
                x: this.0.cframe.position.x,
                y: this.0.cframe.position.y,
                z: this.0.cframe.position.z,
            })
        });
        fields.add_field_method_get("CFrame", |_, this| Ok(this.0.cframe));
        fields.add_field_method_get("Size", |_, this| Ok(this.0.size));
        fields.add_field_method_get("Color", |_, this| Ok(this.0.color));
        fields.add_field_method_get("Transparency", |_, this| Ok(this.0.transparency));
        fields.add_field_method_get("Touched", |_, this| {
            Ok(LuaSignal {
                id: this.0.touched_signal_id,
            })
        });
        fields.add_field_method_get("Parent", |lua, this| {
            if let Some(parent_handle) = this.0.base.parent_handle {
                let instances_map: mlua::Table = lua.named_registry_value("__instance_cache")?;
                let parent_userdata: Option<mlua::AnyUserData> =
                    instances_map.get(parent_handle)?;
                Ok(parent_userdata)
            } else {
                Ok(None)
            }
        });

        fields.add_field_method_set("Name", |_, this, v: String| {
            this.0.base.set_name(v);
            Ok(())
        });
        fields.add_field_method_set("Position", |_, this, v: LuaVector3| {
            this.0.set_position(v);
            Ok(())
        });
        fields.add_field_method_set("CFrame", |_, this, v: LuaCFrame| {
            this.0.set_cframe(v);
            Ok(())
        });
        fields.add_field_method_set("Size", |_, this, v: LuaVector3| {
            this.0.set_size(v);
            Ok(())
        });
        fields.add_field_method_set("Color", |_, this, c: LuaColor3| {
            this.0.set_color(c);
            Ok(())
        });
        fields.add_field_method_set("Transparency", |_, this, t: f32| {
            this.0.set_transparency(t);
            Ok(())
        });
        fields.add_field_method_set("Parent", |_, this, parent: Option<mlua::AnyUserData>| {
            this.0.base.set_parent(parent);
            Ok(())
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.0.base.name.clone()));
        methods.add_method("Destroy", |_, this, ()| {
            this.0.destroy();
            Ok(())
        });
    }
}

pub struct PartModule;

impl LuaModule for PartModule {
    fn name() -> &'static str {
        "Part"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_cache, ()| {
                let handle = next_handle();
                let touched_signal = LuaSignal::new(lua_cache)?;

                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let mat = w
                        .resource_mut::<Assets<StandardMaterial>>()
                        .add(StandardMaterial {
                            base_color: Color::srgb(0.8, 0.8, 0.8),
                            ..default()
                        });
                    let mesh = w.resource_mut::<Assets<Mesh>>().add(Cuboid::default());
                    let entity = w
                        .spawn((
                            Mesh3d(mesh),
                            MeshMaterial3d(mat.clone()),
                            Transform::default(),
                            LuauHandle(handle),
                            Visibility::Hidden,
                        ))
                        .id();
                    w.resource_mut::<HandleMap>()
                        .insert(handle, entity, Some(mat));
                }));

                let part = LuaPart(BasePartData::new(handle, q.clone(), touched_signal.id));
                let userdata = lua_cache.clone().create_userdata(part)?;

                if let Ok(cache) = lua_cache.named_registry_value::<mlua::Table>("__instance_cache")
                {
                    let _ = cache.set(handle, userdata.clone());
                }
                Ok(userdata)
            })?,
        )?;
        lua.globals().set("Part", t)
    }
}
