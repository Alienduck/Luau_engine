use bevy::prelude::*;
use engine_core::input::{ActionMap, BoundKey};
use luau_runtime::{bridge::queue::LuaQueue, registry::LuaModule};
use mlua::{Lua, UserData, UserDataMethods};
use std::sync::{Arc, Mutex};

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

pub struct ContextActionService {
    pub queue: Arc<Mutex<Vec<InputAction>>>,
}

impl UserData for ContextActionService {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "BindAction",
            |_, this, (action, device, name): (String, String, String)| {
                let key = parse_binding(&device, &name);
                this.queue
                    .lock()
                    .unwrap()
                    .push(InputAction::Bind { action, key });
                Ok(())
            },
        );
        methods.add_method("UnbindAction", |_, this, action: String| {
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
    fn register(lua: &Lua, _queue: &LuaQueue) -> mlua::Result<()> {
        let queue: Arc<Mutex<Vec<InputAction>>> = Arc::new(Mutex::new(Vec::new()));
        lua.set_named_registry_value(
            "__input_queue",
            lua.create_userdata(InputQueueHolder(queue.clone()))?,
        )?;
        lua.globals().set(
            "ContextActionService",
            lua.create_userdata(ContextActionService { queue })?,
        )
    }
}

pub struct InputQueueHolder(pub Arc<Mutex<Vec<InputAction>>>);
impl UserData for InputQueueHolder {}

// L'ancien parseur de la caméra est déplacé ici !
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
