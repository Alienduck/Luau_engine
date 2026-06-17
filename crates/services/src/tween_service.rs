use bevy::prelude::*;
use luau_classes::types::{
    tween_core::{TweenableValue, calculate_alpha},
    tween_info::LuaTweenInfo,
};
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule, vm::LuaVm};
use mlua::{Lua, ObjectLike, UserData, UserDataMethods};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TWEEN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(PartialEq)]
pub enum PlayState {
    Playing,
    Paused,
    Completed,
}

pub struct ActiveTween {
    pub instance_key: mlua::RegistryKey,
    pub properties: Vec<(String, TweenableValue, TweenableValue)>,
    pub info: LuaTweenInfo,
    pub elapsed: f32,
    pub state: PlayState,
}

#[derive(Default)]
pub struct TweenEngine {
    pub active_tweens: HashMap<u64, ActiveTween>,
}

pub fn process_tweens_system(vm: NonSend<LuaVm>, time: Res<Time>) {
    let dt = time.delta_secs();
    let lua = &vm.lua;

    let Some(mut engine) = lua.app_data_mut::<TweenEngine>() else {
        return;
    };

    let mut updates = Vec::new();

    for (_, tween) in engine.active_tweens.iter_mut() {
        if tween.state != PlayState::Playing {
            continue;
        }

        tween.elapsed += dt;
        let is_finished = tween.elapsed >= tween.info.time;

        let alpha = calculate_alpha(
            tween.elapsed,
            tween.info.time,
            tween.info.easing_style,
            tween.info.easing_direction,
        );

        if let Ok(instance) = lua.registry_value::<mlua::AnyUserData>(&tween.instance_key) {
            for (prop_name, start_val, end_val) in &tween.properties {
                let current_val = start_val.lerp(end_val, alpha);
                updates.push((instance.clone(), prop_name.clone(), current_val));
            }
        }

        if is_finished {
            tween.state = PlayState::Completed;
        }
    }

    engine
        .active_tweens
        .retain(|_, t| t.state != PlayState::Completed);

    drop(engine);

    for (instance, prop_name, current_val) in updates {
        match current_val.into_lua(lua) {
            Ok(lua_val) => {
                if let Err(e) = instance.set(prop_name.clone(), lua_val) {
                    println!(
                        "[TweenService] Erreur lors de l'application de '{}': {}",
                        prop_name, e
                    );
                }
            }
            Err(e) => println!("[TweenService] Erreur de conversion Rust -> Lua: {}", e),
        }
    }
}

#[derive(Clone)]
pub struct LuaTween {
    pub id: u64,
}

impl UserData for LuaTween {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Play", |lua, this, ()| {
            if let Some(mut engine) = lua.app_data_mut::<TweenEngine>() {
                if let Some(tween) = engine.active_tweens.get_mut(&this.id) {
                    tween.state = PlayState::Playing;
                } else {
                    println!(
                        "[TweenService] Erreur: Impossible de jouer le Tween {} (introuvable)",
                        this.id
                    );
                }
            }
            Ok(())
        });

        methods.add_method("Pause", |lua, this, ()| {
            if let Some(mut engine) = lua.app_data_mut::<TweenEngine>() {
                if let Some(tween) = engine.active_tweens.get_mut(&this.id) {
                    tween.state = PlayState::Paused;
                }
            }
            Ok(())
        });

        methods.add_method("Cancel", |lua, this, ()| {
            if let Some(mut engine) = lua.app_data_mut::<TweenEngine>() {
                engine.active_tweens.remove(&this.id);
            }
            Ok(())
        });
    }
}

pub struct TweenServiceModule;

impl LuaModule for TweenServiceModule {
    fn name() -> &'static str {
        "TweenService"
    }
    fn register(lua: &Lua, _engine: &EngineQueue) -> mlua::Result<()> {
        let ts = lua.create_table()?;

        ts.set(
            "Create",
            lua.create_function(
                |lua,
                 (_, instance, info_ud, properties): (
                    mlua::Value,
                    mlua::AnyUserData,
                    mlua::AnyUserData,
                    mlua::Table,
                )| {
                    let info = info_ud.borrow::<LuaTweenInfo>()?.clone();

                    let mut props_to_animate = Vec::new();

                    for pair in properties.pairs::<String, mlua::Value>() {
                        match pair {
                            Ok((key, target_value)) => {
                                match instance.get::<mlua::Value>(key.clone()) {
                                    Ok(start_value) => {
                                        let parsed_start = TweenableValue::from_lua(start_value);
                                        let parsed_end = TweenableValue::from_lua(target_value);

                                        match (parsed_start, parsed_end) {
                                            (Some(s), Some(e)) => props_to_animate.push((key, s, e)),
                                            (None, _) => println!("[TweenService] Attention: Type de départ non reconnu pour '{}'", key),
                                            (_, None) => println!("[TweenService] Attention: Type d'arrivée non reconnu pour '{}'", key),
                                        }
                                    }
                                    Err(e) => println!(
                                        "[TweenService] Attention: Propriété '{}' introuvable sur l'instance ! ({})",
                                        key, e
                                    ),
                                }
                            }
                            Err(e) => println!(
                                "[TweenService] Erreur lors du parcours des propriétés: {}",
                                e
                            ),
                        }
                    }

                    if props_to_animate.is_empty() {
                        println!("[TweenService] AVERTISSEMENT: Tween créé sans aucune propriété valide !");
                    }

                    let id = NEXT_TWEEN_ID.fetch_add(1, Ordering::Relaxed);
                    let instance_key = lua.create_registry_value(instance)?;

                    let active_tween = ActiveTween {
                        instance_key,
                        properties: props_to_animate,
                        info,
                        elapsed: 0.0,
                        state: PlayState::Paused,
                    };

                    if let Some(mut engine) = lua.app_data_mut::<TweenEngine>() {
                        engine.active_tweens.insert(id, active_tween);
                    } else {
                        println!(
                            "[TweenService] CRITIQUE: TweenEngine absent de la mémoire Luau !"
                        );
                    }

                    Ok(LuaTween { id })
                },
            )?,
        )?;

        lua.globals().set("TweenService", ts)?;
        Ok(())
    }
}
