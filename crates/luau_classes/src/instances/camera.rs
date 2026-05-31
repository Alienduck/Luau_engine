use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use engine_core::input::ActionMap;
use luau_runtime::{bridge::queue::LuaQueue, registry::LuaModule};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};
use std::f32::consts::FRAC_PI_2;
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────
// Bevy component
// ─────────────────────────────────────────────

/// The smart camera component. Attach to the camera entity.
#[derive(Component)]
pub struct SmartCamera {
    pub subject: Option<Entity>,
    pub first_person: bool,
    pub distance: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub fov: f32,
    pub fp_offset: Vec3,
    pub mouse_locked: bool,
    pub look_action: String,
    pub lock_action: String,
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
            fov: std::f32::consts::FRAC_PI_3,
            fp_offset: Vec3::new(0.0, 1.6, 0.0),
            mouse_locked: false,
            look_action: "CameraLook".into(),
            lock_action: "CameraToggleLock".into(),
        }
    }
}

// ─────────────────────────────────────────────
// Bevy systems
// ─────────────────────────────────────────────

pub fn camera_mouse_look(
    mut motion: MessageReader<MouseMotion>,
    mut query: Query<&mut SmartCamera>,
    action_map: Res<ActionMap>,
) {
    let Ok(mut cam) = query.single_mut() else {
        return;
    };

    // Vérification propre via le contexte
    let look_held = action_map.is_pressed(&cam.look_action);
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
    action_map: Res<ActionMap>,
) {
    let Ok(mut cam) = query.single_mut() else {
        return;
    };
    if action_map.just_pressed(&cam.lock_action) {
        cam.mouse_locked = !cam.mouse_locked;
        cursor.grab_mode = if cam.mouse_locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !cam.mouse_locked;
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
        }
    }
}

// ─────────────────────────────────────────────
// Luau UserData handle
// ─────────────────────────────────────────────

pub struct LuaCamera {
    pub queue: Arc<Mutex<Vec<CameraCommand>>>,
    pub first_person: bool,
    pub distance: f32,
    pub sensitivity: f32,
    pub fov: f32,
}

impl LuaCamera {
    fn default(queue: Arc<Mutex<Vec<CameraCommand>>>) -> Self {
        LuaCamera {
            queue,
            first_person: false,
            distance: 8.0,
            sensitivity: 1.0,
            fov: 70.0,
        }
    }
}

impl UserData for LuaCamera {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("FirstPerson", |_, this| Ok(this.first_person));
        fields.add_field_method_get("Distance", |_, this| Ok(this.distance));
        fields.add_field_method_get("Sensitivity", |_, this| Ok(this.sensitivity));
        fields.add_field_method_get("Fov", |_, this| Ok(this.fov));

        fields.add_field_method_set("FirstPerson", |_, this, v: bool| {
            this.first_person = v;
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetFirstPerson(v));
            Ok(())
        });
        fields.add_field_method_set("Distance", |_, this, v: f32| {
            this.distance = v;
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetDistance(v));
            Ok(())
        });
        fields.add_field_method_set("Sensitivity", |_, this, v: f32| {
            this.sensitivity = v;
            this.queue
                .lock()
                .unwrap()
                .push(CameraCommand::SetSensitivity(v));
            Ok(())
        });
        fields.add_field_method_set("Fov", |_, this, v: f32| {
            this.fov = v;
            this.queue.lock().unwrap().push(CameraCommand::SetFov(v));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetSubject", |_, this, part: mlua::AnyUserData| {
            let handle = part.borrow::<crate::instances::part::LuaPart>()?.0.handle;
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
        let cam_queue: Arc<Mutex<Vec<CameraCommand>>> = Arc::new(Mutex::new(Vec::new()));

        lua.set_named_registry_value(
            "__camera_queue",
            lua.create_userdata(CameraQueueHolder(cam_queue.clone()))?,
        )?;

        let cam = LuaCamera::default(cam_queue);
        lua.globals().set("Camera", lua.create_userdata(cam)?)
    }
}

/// Thin holder so we can store the Arc in the Lua registry.
pub struct CameraQueueHolder(pub Arc<Mutex<Vec<CameraCommand>>>);
impl UserData for CameraQueueHolder {}
