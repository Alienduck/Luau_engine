use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use engine_core::input::ActionMap;
use luau_runtime::{
    bridge::{handle::HandleMap, queue::EngineQueue},
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};
use std::f32::consts::FRAC_PI_2;
use std::sync::{Arc, Mutex};

use crate::types::cframe::LuaCFrame;

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
            fp_offset: Vec3::new(0.0, 0.0, 0.0),
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

#[derive(Resource, Clone, Default)]
pub struct CameraCFrame(pub Arc<Mutex<LuaCFrame>>);

pub struct CameraCFrameHolder(pub Arc<Mutex<LuaCFrame>>);
impl UserData for CameraCFrameHolder {}

pub fn camera_update_transform(
    mut cam_query: Query<(&SmartCamera, &mut Transform, &mut Projection)>,
    subject_query: Query<&Transform, Without<SmartCamera>>,
    cframe_sync: Res<CameraCFrame>,
) {
    let Ok((cam, mut cam_tf, mut proj)) = cam_query.single_mut() else {
        return;
    };
    if let Projection::Perspective(ref mut p) = *proj {
        p.fov = cam.fov;
    }
    if let Some(subject) = cam.subject {
        if let Ok(subject_tf) = subject_query.get(subject) {
            let rot = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
            if cam.first_person {
                cam_tf.translation = subject_tf.translation + cam.fp_offset;
            } else {
                let offset = rot * Vec3::new(0.0, 0.0, cam.distance);
                cam_tf.translation = subject_tf.translation + Vec3::Y + offset;
            }
            cam_tf.rotation = rot;
        }
    }

    let mut s = cframe_sync.0.lock().unwrap();
    s.position = cam_tf.translation;
    s.rotation = cam_tf.rotation;
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
// Luau UserData handle
// ─────────────────────────────────────────────

pub struct LuaCamera {
    pub queue: EngineQueue,
    pub cframe: Arc<Mutex<LuaCFrame>>,
    pub first_person: bool,
    pub distance: f32,
    pub sensitivity: f32,
    pub fov: f32,
}

impl LuaCamera {
    fn default(queue: EngineQueue, cframe: Arc<Mutex<LuaCFrame>>) -> Self {
        LuaCamera {
            queue,
            cframe,
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
        fields.add_field_method_get("CFrame", |_, this| Ok(this.cframe.lock().unwrap().clone()));

        fields.add_field_method_set("FirstPerson", |_, this, v: bool| {
            this.first_person = v;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let mut q = w.query::<&mut SmartCamera>();
                    if let Ok(mut cam) = q.single_mut(w) {
                        cam.first_person = v;
                    }
                }));
            Ok(())
        });
        fields.add_field_method_set("Distance", |_, this, v: f32| {
            this.distance = v;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let mut q = w.query::<&mut SmartCamera>();
                    if let Ok(mut cam) = q.single_mut(w) {
                        cam.distance = v
                    }
                }));
            Ok(())
        });
        fields.add_field_method_set("Sensitivity", |_, this, v: f32| {
            this.sensitivity = v;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let mut q = w.query::<&mut SmartCamera>();
                    if let Ok(mut cam) = q.single_mut(w) {
                        cam.sensitivity = v;
                    }
                }));
            Ok(())
        });
        fields.add_field_method_set("Fov", |_, this, v: f32| {
            this.fov = v;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let mut q = w.query::<&mut SmartCamera>();
                    if let Ok(mut cam) = q.single_mut(w) {
                        cam.fov = v;
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("SetSubject", |_, this, part: mlua::AnyUserData| {
            let handle = part
                .borrow::<crate::instances::part::LuaPart>()?
                .0
                .base
                .handle;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let mut q = w.query::<&mut SmartCamera>();
                    let entity = w.resource::<HandleMap>().get_entity(handle);
                    if let Ok(mut cam) = q.single_mut(w) {
                        cam.subject = entity;
                    }
                }));

            Ok(())
        });
        methods.add_method("ClearSubject", |_, this, ()| {
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let mut q = w.query::<&mut SmartCamera>();
                    if let Ok(mut cam) = q.single_mut(w) {
                        cam.subject = None;
                    }
                }));
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

    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let cframe = Arc::new(Mutex::new(LuaCFrame::default()));
        lua.set_named_registry_value(
            "__camera_cframe",
            lua.create_userdata(CameraCFrameHolder(cframe.clone()))?,
        )?;

        let cam = LuaCamera::default(queue.clone(), cframe);
        lua.globals().set("Camera", lua.create_userdata(cam)?)
    }
}
