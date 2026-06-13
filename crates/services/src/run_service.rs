use bevy::prelude::*;
use luau_classes::types::signal::LuaSignal;
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule, vm::LuaVm};
use mlua::{Lua, UserData, UserDataFields};

/// Luau-facing `RunService` singleton.
///
/// Exposes the `RenderStepped` signal, fired once per frame with `dt` as its
/// argument.  Mirrors the Roblox `RunService.RenderStepped` event.
pub struct RunService {
    pub render_stepped_id: u64,
}

impl UserData for RunService {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("RenderStepped", |_, this| {
            Ok(LuaSignal {
                id: this.render_stepped_id,
            })
        });
    }
}

pub struct RunServiceModule;

impl LuaModule for RunServiceModule {
    fn name() -> &'static str {
        "RunService"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let render_stepped = LuaSignal::new(lua)?;
        lua.set_named_registry_value("__rs_render_stepped", render_stepped.id)?;
        lua.globals().set(
            "RunService",
            lua.create_userdata(RunService {
                render_stepped_id: render_stepped.id,
            })?,
        )
    }
}

/// Bevy system — fires `RunService.RenderStepped` with the frame delta time.
///
/// Must run every frame, after the engine queue is processed.
pub fn trigger_run_service(vm: NonSend<LuaVm>, time: Res<Time>) {
    let lua = vm.lua();
    if let Ok(id) = lua.named_registry_value::<u64>("__rs_render_stepped") {
        let _ = LuaSignal { id }.fire(lua, time.delta().as_secs_f64());
    }
}
