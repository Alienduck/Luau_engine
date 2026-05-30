use super::base_part::BasePartData;
use crate::types::{color3::LuaColor3, vector3::LuaVector3};
use luau_runtime::{
    bridge::{
        handle::next_handle,
        queue::{LuaCommand, LuaQueue},
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};

pub struct LuaPart(pub BasePartData);

impl UserData for LuaPart {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        // Getters — read from local cache, always accurate
        fields.add_field_method_get("Position", |_, this| Ok(this.0.position));
        fields.add_field_method_get("Size", |_, this| Ok(this.0.size));
        fields.add_field_method_get("Color", |_, this| Ok(this.0.color));

        // Setters — update cache + push command
        fields.add_field_method_set("Position", |_, this, v: LuaVector3| {
            this.0.set_position(v);
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
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
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

    fn register(lua: &Lua, queue: &LuaQueue) -> mlua::Result<()> {
        let q = queue.0.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |_, ()| {
                let handle = next_handle();
                q.lock().unwrap().push(LuaCommand::SpawnPart {
                    handle,
                    position: bevy::math::Vec3::ZERO,
                    size: bevy::math::Vec3::ONE,
                    color: bevy::prelude::Color::srgb(0.8, 0.8, 0.8),
                });
                Ok(LuaPart(BasePartData::new(handle, q.clone())))
            })?,
        )?;
        lua.globals().set("Part", t)
    }
}
