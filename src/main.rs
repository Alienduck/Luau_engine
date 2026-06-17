use bevy::{
    core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*,
    render::view::Hdr, window::WindowMode,
};
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use engine_core::input::{ActionMap, update_action_states};
use luau_classes::{
    instances::{
        base_part::process_collisions,
        bloom_effect::BloomEffectModule,
        camera::{CameraCFrame, CameraCFrameHolder, CameraModule, SmartCamera, SmartCameraPlugin},
        lighting::{atmosphere::AtmosphereModule, sky::SkyModule},
        part::PartModule,
        physics::{collider::ColliderModule, rigidbody::RigidbodyModule},
        ui::{
            frame::FrameModule, image_button::ImageButtonModule, image_label::ImageLabelModule,
            screen_gui::ScreenGuiModule, text_button::TextButtonModule,
            text_label::TextLabelModule, ui_interactions::process_button_interactions,
        },
        workspace::{WorkspaceModule, sync_dormancy_system},
    },
    types::{
        cframe::CFrameModule, color3::Color3Module, udim2::Udim2Module, vector2::Vector2Module,
        vector3::Vector3Module,
    },
};
use luau_runtime::{
    bridge::{
        handle::HandleMap,
        queue::{EngineQueue, process_engine_queue},
    },
    registry::LuaModule,
    scheduler::{LuaScheduler, tick_scheduler},
    vm::LuaVm,
};
use services::{
    lighting::{
        LightingModule, sync_atmosphere_system, sync_post_processing_system, sync_sky_system,
    },
    run_service::{RunServiceModule, trigger_run_service},
    user_input::{UserInputModule, trigger_user_input},
};
use std::fs;

fn main() {
    let queue = EngineQueue::default();
    let vm = LuaVm::new().expect("failed to create Lua VM");
    let mut scheduler = LuaScheduler::new();

    // Make the queue accessible from Lua closures that need it (e.g. Destroy).
    vm.lua().set_app_data(queue.clone());

    register_all(vm.lua(), &queue);

    // Extract the shared CFrame Arc from the Lua registry so Bevy can write
    // to it each frame without going through the queue.
    let cam_cframe: CameraCFrame = {
        let holder = vm
            .lua()
            .named_registry_value::<mlua::AnyUserData>("__camera_cframe")
            .expect("CameraModule must be registered before extracting CFrame");
        let arc = holder.borrow::<CameraCFrameHolder>().unwrap().0.clone();
        CameraCFrame(arc)
    };

    // Load and spawn the entry-point script as the first scheduled coroutine.
    let script =
        fs::read_to_string("scripts/startup.luau").expect("scripts/startup.luau not found");
    let thread = vm
        .lua()
        .create_thread(
            vm.lua()
                .load(&script)
                .set_name("startup")
                .into_function()
                .expect("startup.luau must be valid Luau"),
        )
        .unwrap();
    scheduler.spawn(thread);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luau Engine".into(),
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(SmartCameraPlugin)
        .insert_resource(queue)
        .insert_resource(HandleMap::default())
        .insert_resource(ActionMap::default())
        .insert_resource(cam_cframe)
        .insert_non_send_resource(vm)
        .insert_non_send_resource(scheduler)
        .add_systems(
            PreUpdate,
            // Input states must be refreshed before collision processing so
            // that action-map queries in the same frame see correct values.
            (update_action_states, process_collisions).chain(),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                tick_scheduler,
                process_engine_queue,
                trigger_user_input,
                trigger_run_service,
                sync_dormancy_system,
                sync_post_processing_system,
                sync_sky_system,
                sync_atmosphere_system,
                process_button_interactions,
            )
                .chain(),
        )
        .run();
}

/// Registers all Luau modules (classes and services) into the VM.
///
/// Order matters: modules that other modules depend on at registration time
/// (e.g. `WorkspaceModule` needs the instance cache to exist) must come after
/// the VM is fully initialised but before any script runs.
fn register_all(lua: &mlua::Lua, queue: &EngineQueue) {
    let modules: &[(&str, fn(&mlua::Lua, &EngineQueue) -> mlua::Result<()>)] = &[
        (Vector2Module::name(), Vector2Module::register),
        (Vector3Module::name(), Vector3Module::register),
        (Color3Module::name(), Color3Module::register),
        (CFrameModule::name(), CFrameModule::register),
        (PartModule::name(), PartModule::register),
        (CameraModule::name(), CameraModule::register),
        (UserInputModule::name(), UserInputModule::register),
        (RigidbodyModule::name(), RigidbodyModule::register),
        (ColliderModule::name(), ColliderModule::register),
        (RunServiceModule::name(), RunServiceModule::register),
        (ScreenGuiModule::name(), ScreenGuiModule::register),
        (FrameModule::name(), FrameModule::register),
        (TextLabelModule::name(), TextLabelModule::register),
        (ImageLabelModule::name(), ImageLabelModule::register),
        (TextButtonModule::name(), TextButtonModule::register),
        (ImageButtonModule::name(), ImageButtonModule::register),
        (BloomEffectModule::name(), BloomEffectModule::register),
        (SkyModule::name(), SkyModule::register),
        (AtmosphereModule::name(), AtmosphereModule::register),
        (Udim2Module::name(), Udim2Module::register),
        (WorkspaceModule::name(), WorkspaceModule::register),
        (LightingModule::name(), LightingModule::register),
    ];
    for (name, register) in modules {
        if let Err(e) = register(lua, queue) {
            panic!("failed to register Lua module '{}': {}", name, e);
        }
    }
}

/// Spawns the camera and directional light entities at startup.
fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Hdr,
        Camera::default(),
        Camera3d::default(),
        // Camera2d::default(),
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        SmartCamera::default(),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
