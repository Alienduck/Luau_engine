use bevy::prelude::*;
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule, vm::LuaVm};
use mlua::{Lua, UserData, UserDataMethods};

pub struct RunService;

impl UserData for RunService {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "BindToRenderStep",
            |lua, _, (name, _priority, callback): (String, i32, mlua::Function)| {
                let table = lua.named_registry_value::<mlua::Table>("__runservice_callbacks")?;
                table.set(name, callback)?;
                Ok(())
            },
        );
        methods.add_method("UnbindFromRenderStep", |lua, _, name: String| {
            if let Ok(table) = lua.named_registry_value::<mlua::Table>("__runservice_callbacks") {
                let _ = table.set(name, mlua::Value::Nil);
            }
            Ok(())
        });
    }
}

pub struct RunServiceModule;

impl LuaModule for RunServiceModule {
    fn name() -> &'static str {
        "RunService"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        lua.set_named_registry_value("__runservice_callbacks", lua.create_table()?)?;
        lua.globals()
            .set("RunService", lua.create_userdata(RunService)?)
    }
}

pub fn trigger_run_service(vm: NonSend<LuaVm>, time: Res<Time>) {
    let Ok(table) = vm
        .lua()
        .named_registry_value::<mlua::Table>("__runservice_callbacks")
    else {
        return;
    };

    let dt = time.delta().as_secs_f64();

    for pair in table.pairs::<String, mlua::Function>() {
        if let Ok((name, callback)) = pair {
            if let Err(e) = callback.call::<()>(dt) {
                log::error!("[RunService] error in '{}': {}", name, e);
            }
        }
    }
}
