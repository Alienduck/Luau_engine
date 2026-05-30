use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use luau_runtime::{bridge::queue::LuaQueue, registry::LuaModule};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};
use std::f32::consts::FRAC_PI_2;
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────
// Input binding types
// ─────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundKey {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

/// Configurable bindings for camera controls, set from Luau.
#[derive(Clone, Debug)]
pub struct CameraBindings {
    /// Hold this to orbit / look (3rd person) or look freely (1st person).
    /// Default: right mouse button.
    pub look: BoundKey,
    /// Toggle mouse lock (grab cursor). Default: Left Shift.
    pub toggle_lock: BoundKey,
}

impl Default for CameraBindings {
    fn default() -> Self {
        Self {
            look: BoundKey::Mouse(MouseButton::Right),
            toggle_lock: BoundKey::Keyboard(KeyCode::ShiftLeft),
        }
    }
}

// ─────────────────────────────────────────────
// Bevy component
// ─────────────────────────────────────────────

/// The smart camera component. Attach to the camera entity.
#[derive(Component)]
pub struct SmartCamera {
    /// Entity to follow (must have a Transform).
    pub subject: Option<Entity>,
    /// true = first person, false = third person.
    pub first_person: bool,
    /// Third-person orbit distance.
    pub distance: f32,
    /// Mouse sensitivity multiplier.
    pub sensitivity: f32,
    /// Vertical pitch (radians).
    pub pitch: f32,
    /// Horizontal yaw (radians).
    pub yaw: f32,
    /// Vertical FOV in radians.
    pub fov: f32,
    /// Offset applied in first-person mode (eye position relative to subject).
    pub fp_offset: Vec3,
    /// Mouse is currently grabbed.
    pub mouse_locked: bool,
    /// Input bindings, configurable from Luau.
    pub bindings: CameraBindings,
}

impl Default for SmartCamera {
    fn default() -> Self {
        Self {
            subject: None,
            first_person: false,
            distance: 8.0,
            sensitivity: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            fov: std::f32::consts::FRAC_PI_3, // 60°
            fp_offset: Vec3::new(0.0, 1.6, 0.0),
            mouse_locked: false,
            bindings: CameraBindings::default(),
        }
    }
}

// ─────────────────────────────────────────────
// Bevy systems
// ─────────────────────────────────────────────

fn is_pressed(
    key: BoundKey,
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    match key {
        BoundKey::Keyboard(k) => keys.pressed(k),
        BoundKey::Mouse(b) => mouse.pressed(b),
    }
}

fn just_pressed(
    key: BoundKey,
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
) -> bool {
    match key {
        BoundKey::Keyboard(k) => keys.just_pressed(k),
        BoundKey::Mouse(b) => mouse.just_pressed(b),
    }
}

pub fn camera_mouse_look(
    mut motion: MessageReader<MouseMotion>,
    mut query: Query<&mut SmartCamera>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let Ok(mut cam) = query.single_mut() else {
        return;
    };

    // Only rotate when the look key is held, or when mouse is locked
    let look_held = is_pressed(cam.bindings.look, &keys, &mouse);
    if !look_held && !cam.mouse_locked {
        motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion.read() {
        delta += ev.delta;
    }

    if delta != Vec2::ZERO {
        let s = cam.sensitivity * 0.003;
        cam.yaw -= delta.x * s;
        cam.pitch -= delta.y * s;
        cam.pitch = cam.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }
}

pub fn camera_toggle_lock(
    mut query: Query<&mut SmartCamera>,
    mut cursor: Single<&mut CursorOptions>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let Ok(mut cam) = query.single_mut() else {
        return;
    };
    if just_pressed(cam.bindings.toggle_lock, &keys, &mouse) {
        cam.mouse_locked = !cam.mouse_locked;
        if cam.mouse_locked {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        } else {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}

pub fn camera_update_transform(
    mut cam_query: Query<(&SmartCamera, &mut Transform, &mut Projection)>,
    subject_query: Query<&Transform, Without<SmartCamera>>,
) {
    let Ok((cam, mut cam_tf, mut proj)) = cam_query.single_mut() else {
        return;
    };

    // Sync FOV
    if let Projection::Perspective(ref mut p) = *proj {
        p.fov = cam.fov;
    }

    let Some(subject) = cam.subject else {
        return;
    };
    let Ok(subject_tf) = subject_query.get(subject) else {
        return;
    };

    let rot = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);

    if cam.first_person {
        cam_tf.translation = subject_tf.translation + cam.fp_offset;
    } else {
        let offset = rot * Vec3::new(0.0, 0.0, cam.distance);
        cam_tf.translation = subject_tf.translation + Vec3::Y + offset;
    }
    cam_tf.rotation = rot;
}

/// Bevy plugin — adds all camera systems.
pub struct SmartCameraPlugin;

impl Plugin for SmartCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_mouse_look, camera_toggle_lock).chain())
            .add_systems(PostUpdate, camera_update_transform);
    }
}

// ─────────────────────────────────────────────
// Luau-side handle
// ─────────────────────────────────────────────

/// Commands the camera system can receive from Luau scripts.
pub enum CameraCommand {
    SetFirstPerson(bool),
    SetDistance(f32),
    SetSensitivity(f32),
    SetFov(f32),
    SetSubject(u64), // handle → resolved to Entity in the system
    ClearSubject,
    SetLookBinding { keyboard: bool, code: u32 },
    SetLockBinding { keyboard: bool, code: u32 },
}

/// Shared queue for camera commands (parallel to LuaQueue for world commands).
#[derive(Resource, Clone, Default)]
pub struct CameraQueue(pub Arc<Mutex<Vec<CameraCommand>>>);

/// System that drains CameraQueue and applies changes to SmartCamera.
pub fn process_camera_queue(
    queue: Res<CameraQueue>,
    handle_map: Res<luau_runtime::bridge::handle::HandleMap>,
    mut query: Query<&mut SmartCamera>,
) {
    let Ok(mut cam) = query.single_mut() else {
        return;
    };
    let commands: Vec<CameraCommand> = queue.0.lock().unwrap().drain(..).collect();

    for cmd in commands {
        match cmd {
            CameraCommand::SetFirstPerson(v) => cam.first_person = v,
            CameraCommand::SetDistance(v) => cam.distance = v.max(0.1),
            CameraCommand::SetSensitivity(v) => cam.sensitivity = v,
            CameraCommand::SetFov(deg) => cam.fov = deg.to_radians(),
            CameraCommand::SetSubject(h) => {
                cam.subject = handle_map.get_entity(h);
            }
            CameraCommand::ClearSubject => cam.subject = None,
            CameraCommand::SetLookBinding { keyboard, code } => {
                cam.bindings.look = decode_binding(keyboard, code);
            }
            CameraCommand::SetLockBinding { keyboard, code } => {
                cam.bindings.toggle_lock = decode_binding(keyboard, code);
            }
        }
    }
}

fn decode_binding(keyboard: bool, code: u32) -> BoundKey {
    if keyboard {
        // Map a small subset of common keys by scancode-like integer.
        // Luau side uses string names; we resolve them in the Lua register fn.
        BoundKey::Keyboard(u32_to_keycode(code))
    } else {
        BoundKey::Mouse(u32_to_mouse_button(code))
    }
}

fn u32_to_keycode(v: u32) -> KeyCode {
    match v {
        0 => KeyCode::KeyW,
        1 => KeyCode::KeyA,
        2 => KeyCode::KeyS,
        3 => KeyCode::KeyD,
        4 => KeyCode::ShiftLeft,
        5 => KeyCode::ShiftRight,
        6 => KeyCode::ControlLeft,
        7 => KeyCode::Space,
        8 => KeyCode::KeyE,
        9 => KeyCode::KeyQ,
        10 => KeyCode::KeyF,
        11 => KeyCode::KeyR,
        12 => KeyCode::KeyG,
        13 => KeyCode::Escape,
        _ => KeyCode::ShiftLeft,
    }
}

fn u32_to_mouse_button(v: u32) -> MouseButton {
    match v {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        _ => MouseButton::Right,
    }
}

// ─────────────────────────────────────────────
// Luau UserData handle
// ─────────────────────────────────────────────

pub struct LuaCamera {
    pub queue: Arc<Mutex<Vec<CameraCommand>>>,
}

impl UserData for LuaCamera {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_set("FirstPerson", |_, this, v: bool| {
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetFirstPerson(v));
            Ok(())
        });
        fields.add_field_method_set("Distance", |_, this, v: f32| {
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetDistance(v));
            Ok(())
        });
        fields.add_field_method_set("Sensitivity", |_, this, v: f32| {
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetSensitivity(v));
            Ok(())
        });
        fields.add_field_method_set("Fov", |_, this, v: f32| {
            this.queue.lock().unwrap().push(CameraCommand::SetFov(v));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // camera:SetSubject(part)  — follow a Part
        methods.add_method("SetSubject", |_, this, handle: u64| {
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetSubject(handle));
            Ok(())
        });
        methods.add_method("ClearSubject", |_, this, ()| {
            this.queue.lock().unwrap().push(CameraCommand::ClearSubject);
            Ok(())
        });

        // camera:SetLookBinding("Mouse", "Right")
        // camera:SetLookBinding("Keyboard", "KeyF")
        methods.add_method(
            "SetLookBinding",
            |_, this, (device, name): (String, String)| {
                let (kb, code) = parse_binding(&device, &name);
                this.queue
                    .lock()
                    .unwrap()
                    .push(CameraCommand::SetLookBinding { keyboard: kb, code });
                Ok(())
            },
        );
        methods.add_method(
            "SetLockBinding",
            |_, this, (device, name): (String, String)| {
                let (kb, code) = parse_binding(&device, &name);
                this.queue
                    .lock()
                    .unwrap()
                    .push(CameraCommand::SetLockBinding { keyboard: kb, code });
                Ok(())
            },
        );
    }
}

fn parse_binding(device: &str, name: &str) -> (bool, u32) {
    if device.eq_ignore_ascii_case("mouse") {
        let code = match name.to_lowercase().as_str() {
            "left" => 0,
            "right" => 1,
            "middle" => 2,
            "back" => 3,
            "forward" => 4,
            _ => 1,
        };
        (false, code)
    } else {
        let code = match name.to_lowercase().as_str() {
            "w" | "keyw" => 0,
            "a" | "keya" => 1,
            "s" | "keys" => 2,
            "d" | "keyd" => 3,
            "shiftleft" | "shift" => 4,
            "shiftright" => 5,
            "controlleft" => 6,
            "space" => 7,
            "e" | "keye" => 8,
            "q" | "keyq" => 9,
            "f" | "keyf" => 10,
            "r" | "keyr" => 11,
            "g" | "keyg" => 12,
            "escape" => 13,
            _ => 4,
        };
        (true, code)
    }
}

// ─────────────────────────────────────────────
// LuaModule — registers `Camera` global
// ─────────────────────────────────────────────

pub struct CameraModule;

impl LuaModule for CameraModule {
    fn name() -> &'static str {
        "Camera"
    }

    fn register(lua: &Lua, _queue: &LuaQueue) -> mlua::Result<()> {
        // CameraQueue is inserted as a Bevy resource separately in main.rs.
        // Here we expose a global `Camera` singleton that wraps it.
        // The queue Arc is passed in via a closure captured at registration time.
        // main.rs must call Camera::init(queue) before registering this module.
        // We store the queue in a Lua registry value so the singleton can share it.
        let cam_queue: Arc<Mutex<Vec<CameraCommand>>> = Arc::new(Mutex::new(Vec::new()));

        // Store the Arc in the Lua registry so main.rs can retrieve it
        lua.set_named_registry_value(
            "__camera_queue",
            lua.create_userdata(CameraQueueHolder(cam_queue.clone()))?,
        )?;

        let cam = LuaCamera { queue: cam_queue };
        lua.globals().set("Camera", lua.create_userdata(cam)?)
    }
}

/// Thin holder so we can store the Arc in the Lua registry.
pub struct CameraQueueHolder(pub Arc<Mutex<Vec<CameraCommand>>>);
impl UserData for CameraQueueHolder {}
