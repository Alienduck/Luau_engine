use bevy::prelude::*;
use luau_classes::types::signal::LuaSignal;
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule, vm::LuaVm};
use mlua::{Lua, UserData, UserDataFields};

/// Luau-facing `UserInputService` singleton.
///
/// Exposes `InputBegan` and `InputEnded` signals, each fired with
/// `(keyName: string, device: string)` arguments.  Key names use Bevy's
/// `KeyCode` debug format (e.g. `"KeyA"`, `"Space"`).
pub struct UserInputService {
    pub began_id: u64,
    pub ended_id: u64,
}

impl UserData for UserInputService {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("InputBegan", |_, this| Ok(LuaSignal { id: this.began_id }));
        fields.add_field_method_get("InputEnded", |_, this| Ok(LuaSignal { id: this.ended_id }));
    }
}

pub struct UserInputModule;

impl LuaModule for UserInputModule {
    fn name() -> &'static str {
        "UserInputService"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let began = LuaSignal::new(lua)?;
        let ended = LuaSignal::new(lua)?;
        lua.set_named_registry_value("__uis_began", began.id)?;
        lua.set_named_registry_value("__uis_ended", ended.id)?;
        lua.globals().set(
            "UserInputService",
            lua.create_userdata(UserInputService {
                began_id: began.id,
                ended_id: ended.id,
            })?,
        )
    }
}

/// Bevy system — fires `InputBegan` / `InputEnded` for all keyboard and mouse
/// events that occurred this frame.
///
/// Note: `KeyCode` is formatted via `{:?}` which produces strings like
/// `"KeyA"`, `"Space"`, `"ArrowLeft"`.  These are stable across Bevy minor
/// versions within 0.18.x but may change on major upgrades.
pub fn trigger_user_input(
    vm: NonSend<LuaVm>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let lua = vm.lua();

    if let Ok(id) = lua.named_registry_value::<u64>("__uis_began") {
        let sig = LuaSignal { id };
        for k in keys.get_just_pressed() {
            let _ = sig.fire(lua, (format!("{:?}", k), "Keyboard"));
        }
        for m in mouse.get_just_pressed() {
            let _ = sig.fire(lua, (format!("{:?}", m), "Mouse"));
        }
    }

    if let Ok(id) = lua.named_registry_value::<u64>("__uis_ended") {
        let sig = LuaSignal { id };
        for k in keys.get_just_released() {
            let _ = sig.fire(lua, (format!("{:?}", k), "Keyboard"));
        }
        for m in mouse.get_just_released() {
            let _ = sig.fire(lua, (format!("{:?}", m), "Mouse"));
        }
    }
}
