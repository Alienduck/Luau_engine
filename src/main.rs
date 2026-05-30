use bevy::prelude::*;
use luau_classes::{
    instances::part::PartModule,
    types::{color3::Color3Module, vector3::Vector3Module},
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
use std::fs;

fn main() {
    let queue = LuaQueue::new();
    let vm = LuaVm::new().expect("failed to create Lua VM");
    let mut scheduler = LuaScheduler::new();

    register_all(vm.lua(), &queue);

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
        .insert_resource(queue)
        .insert_resource(HandleMap::default())
        .insert_non_send_resource(vm)
        .insert_non_send_resource(scheduler)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (tick_scheduler, process_lua_queue).chain())
        .run();
}

fn register_all(lua: &mlua::Lua, queue: &LuaQueue) {
    let modules: &[(&str, fn(&mlua::Lua, &LuaQueue) -> mlua::Result<()>)] = &[
        (Vector3Module::name(), Vector3Module::register),
        (Color3Module::name(), Color3Module::register),
        (PartModule::name(), PartModule::register),
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
        Transform::from_xyz(5.0, 8.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
