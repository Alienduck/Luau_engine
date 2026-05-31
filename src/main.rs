use bevy::prelude::*;
use engine_core::input::{ActionMap, update_action_states};
use luau_classes::{
    instances::{
        camera::{
            CameraModule, CameraQueue, CameraQueueHolder, SmartCamera, SmartCameraPlugin,
            process_camera_queue,
        },
        part::PartModule,
    },
    types::{cframe::CFrameModule, color3::Color3Module, vector3::Vector3Module},
};
use luau_runtime::{
    bridge::{
        handle::HandleMap,
        queue::{LuaQueue, process_lua_queue},
    },
    registry::LuaModule,
    scheduler::{LuaScheduler, tick_scheduler},
    vm::LuaVm,
};
use services::context_action::{
    ContextActionModule, InputQueue, InputQueueHolder, process_input_queue, trigger_context_actions,
};
use std::fs;

fn main() {
    let queue = LuaQueue::new();
    let vm = LuaVm::new().expect("failed to create Lua VM");
    let mut scheduler = LuaScheduler::new();

    register_all(vm.lua(), &queue);

    let cam_queue: CameraQueue = {
        let holder = vm
            .lua()
            .named_registry_value::<mlua::AnyUserData>("__camera_queue")
            .expect("__camera_queue not found in registry");
        let arc = holder.borrow::<CameraQueueHolder>().unwrap().0.clone();
        CameraQueue(arc)
    };

    let input_queue: InputQueue = {
        let holder = vm
            .lua()
            .named_registry_value::<mlua::AnyUserData>("__input_queue")
            .unwrap();
        let arc = holder.borrow::<InputQueueHolder>().unwrap().0.clone();
        InputQueue(arc)
    };

    let script =
        fs::read_to_string("scripts/startup.luau").expect("scripts/startup.luau not found");
    let thread = vm
        .lua()
        .create_thread(
            vm.lua()
                .load(&script)
                .set_name("startup")
                .into_function()
                .unwrap(),
        )
        .unwrap();
    scheduler.spawn(thread);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luau Engine".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SmartCameraPlugin)
        .insert_resource(queue)
        .insert_resource(cam_queue)
        .insert_resource(HandleMap::default())
        .insert_resource(ActionMap::default())
        .insert_resource(input_queue)
        .insert_non_send_resource(vm)
        .insert_non_send_resource(scheduler)
        .add_systems(
            PreUpdate,
            (update_action_states, process_input_queue).chain(),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                tick_scheduler,
                process_lua_queue,
                process_camera_queue,
                trigger_context_actions,
            )
                .chain(),
        )
        .run();
}

fn register_all(lua: &mlua::Lua, queue: &LuaQueue) {
    let modules: &[(&str, fn(&mlua::Lua, &LuaQueue) -> mlua::Result<()>)] = &[
        (Vector3Module::name(), Vector3Module::register),
        (Color3Module::name(), Color3Module::register),
        (CFrameModule::name(), CFrameModule::register),
        (PartModule::name(), PartModule::register),
        (CameraModule::name(), CameraModule::register),
        (ContextActionModule::name(), ContextActionModule::register),
    ];
    for (name, register) in modules {
        if let Err(e) = register(lua, queue) {
            panic!("failed to register Lua module '{}': {}", name, e);
        }
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
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
