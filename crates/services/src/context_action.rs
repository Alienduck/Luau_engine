use bevy::prelude::*;
use engine_core::input::{ActionMap, BoundKey};
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule, vm::LuaVm};
use mlua::{IntoLua, Lua, UserData, UserDataMethods};
use std::sync::{Arc, Mutex};

pub enum InputState {
    Begin,
    End,
}

impl IntoLua for InputState {
    fn into_lua(self, lua: &Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue> {
        match self {
            InputState::Begin => "Begin".into_lua(lua),
            InputState::End => "End".into_lua(lua),
        }
    }
}

impl Into<String> for InputState {
    fn into(self) -> String {
        match self {
            InputState::Begin => "Begin".into(),
            InputState::End => "End".into(),
        }
    }
}

pub enum InputAction {
    Bind { action: String, key: BoundKey },
    Unbind { action: String },
}

#[derive(Resource, Clone, Default)]
pub struct InputQueue(pub Arc<Mutex<Vec<InputAction>>>);

pub fn process_input_queue(queue: Res<InputQueue>, mut action_map: ResMut<ActionMap>) {
    let commands: Vec<InputAction> = queue.0.lock().unwrap().drain(..).collect();
    for cmd in commands {
        match cmd {
            InputAction::Bind { action, key } => {
                action_map.bindings.entry(action).or_default().push(key);
            }
            InputAction::Unbind { action } => {
                action_map.bindings.remove(&action);
            }
        }
    }
}

pub fn trigger_context_actions(vm: NonSend<LuaVm>, action_map: Res<ActionMap>) {
    if action_map.just_pressed_actions.is_empty() && action_map.just_released_actions.is_empty() {
        return;
    }

    let Ok(table) = vm
        .lua()
        .named_registry_value::<mlua::Table>("__context_callbacks")
    else {
        return;
    };

    for action in &action_map.just_pressed_actions {
        if let Ok(callback) = table.get::<mlua::Function>(action.as_str()) {
            if let Err(e) =
                callback.call::<()>((action.clone(), InputState::Begin.into()) as (String, String))
            {
                log::error!("[ContextAction] error in '{}': {}", action, e);
            }
        }
    }

    for action in &action_map.just_released_actions {
        if let Ok(callback) = table.get::<mlua::Function>(action.as_str()) {
            if let Err(e) =
                callback.call::<()>((action.clone(), InputState::End.into()) as (String, String))
            {
                log::error!("[ContextAction] error in '{}': {}", action, e);
            }
        }
    }
}

pub struct ContextActionService {
    pub queue: Arc<Mutex<Vec<InputAction>>>,
}

impl UserData for ContextActionService {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "BindAction",
            |lua, this, (action, callback, device, name): (String, mlua::Function, String, String)| {
                let table = lua.named_registry_value::<mlua::Table>("__context_callbacks")?;
                table.set(action.clone(), callback)?;
                let key = parse_binding(&device, &name);
                this.queue
                    .lock()
                    .unwrap()
                    .push(InputAction::Bind { action, key });
                Ok(())
            },
        );
        methods.add_method("UnbindAction", |lua, this, action: String| {
            if let Ok(table) = lua.named_registry_value::<mlua::Table>("__context_callbacks") {
                let _ = table.set(action.clone(), mlua::Value::Nil);
            }
            this.queue
                .lock()
                .unwrap()
                .push(InputAction::Unbind { action });
            Ok(())
        });
    }
}

pub struct ContextActionModule;

impl LuaModule for ContextActionModule {
    fn name() -> &'static str {
        "ContextActionService"
    }
    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let queue: Arc<Mutex<Vec<InputAction>>> = Arc::new(Mutex::new(Vec::new()));
        lua.set_named_registry_value(
            "__input_queue",
            lua.create_userdata(InputQueueHolder(queue.clone()))?,
        )?;
        lua.set_named_registry_value("__context_callbacks", lua.create_table()?)?;
        lua.globals().set(
            "ContextActionService",
            lua.create_userdata(ContextActionService { queue })?,
        )
    }
}

pub struct InputQueueHolder(pub Arc<Mutex<Vec<InputAction>>>);
impl UserData for InputQueueHolder {}

pub fn parse_binding(device: &str, name: &str) -> BoundKey {
    if device.eq_ignore_ascii_case("mouse") {
        let code = match name.to_lowercase().as_str() {
            "left" => MouseButton::Left,
            "middle" => MouseButton::Middle,
            "back" => MouseButton::Back,
            "forward" => MouseButton::Forward,
            _ => MouseButton::Right,
        };
        BoundKey::Mouse(code)
    } else {
        let code = match name.to_lowercase().as_str() {
            "w" | "keyw" => KeyCode::KeyW,
            "a" | "keya" => KeyCode::KeyA,
            "s" | "keys" => KeyCode::KeyS,
            "d" | "keyd" => KeyCode::KeyD,
            "shiftleft" | "shift" => KeyCode::ShiftLeft,
            "space" => KeyCode::Space,
            "e" | "keye" => KeyCode::KeyE,
            "q" | "keyq" => KeyCode::KeyQ,
            "f" | "keyf" => KeyCode::KeyF,
            "escape" => KeyCode::Escape,
            _ => KeyCode::ShiftLeft,
        };
        BoundKey::Keyboard(code)
    }
}
