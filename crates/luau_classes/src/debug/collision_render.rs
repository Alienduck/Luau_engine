use crate::types::{instance::InstanceData, signal::LuaSignal};
use luau_runtime::{
    bridge::{handle::next_handle, queue::EngineQueue},
    registry::LuaModule,
};
use mlua::{Lua, UserData};

pub struct LuauCollisionRender {
    pub base: InstanceData,
    pub enable: bool,
    pub debug_mode: u8,
}

impl UserData for LuauCollisionRender {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Enable", |_, this| Ok(this.enable));
        fields.add_field_method_set("Enable", |_, this, v: bool| {
            this.enable = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::EnableRenderCollisionDebug {
                    enable: v,
                },
            );
            Ok(())
        });

        fields.add_field_method_get("DebugMode", |_, this| Ok(this.debug_mode));
        fields.add_field_method_set("DebugMode", |_, this, v: u8| {
            this.debug_mode = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetModeRenderCollisionDebug { mode: v },
            );
            Ok(())
        });
    }
}

pub struct CollisionRenderModule;

impl LuaModule for CollisionRenderModule {
    fn name() -> &'static str {
        "CollisionRender"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let handle = next_handle();
        let dsi = LuaSignal::new(lua)?.id;
        let render = LuauCollisionRender {
            base: crate::types::instance::InstanceData::new(
                handle,
                queue.clone(),
                "CollisionRender",
                dsi,
            ),
            enable: false,
            debug_mode: 0,
        };
        let ud = lua.create_userdata(render)?;
        lua.named_registry_value::<mlua::Table>("__instance_cache")?
            .set(handle, ud.clone())?;
        lua.globals().set("CollisionRender", ud)?;
        Ok(())
    }
}
