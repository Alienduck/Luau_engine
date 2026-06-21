use bevy::prelude::*;
use luau_classes::types::{
    signal::LuaSignal,
    tween_core::{TweenableValue, calculate_alpha},
    tween_info::LuaTweenInfo,
};
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule, vm::LuaVm};
use mlua::{Lua, ObjectLike, UserData, UserDataFields, UserDataMethods};
use std::collections::HashMap;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

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
    pub completed_signal_id: u64,
    /// This token allows to know if the Lua object is alive
    pub lua_token: Weak<()>,
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
    let mut completed_signals = Vec::new();

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
            completed_signals.push(tween.completed_signal_id);
        }
    }

    engine
        .active_tweens
        .retain(|_, t| t.state == PlayState::Playing || t.lua_token.strong_count() > 0);

    // Drop the engine so Lua can use it
    drop(engine);

    for (instance, prop_name, current_val) in updates {
        match current_val.into_lua(lua) {
            Ok(lua_val) => {
                if let Err(e) = instance.set(prop_name.clone(), lua_val) {
                    println!("[TweenService] Error while applying '{}': {}", prop_name, e);
                }
            }
            Err(e) => println!("[TweenService] Error while converting Rust -> Lua: {}", e),
        }
    }

    for sig_id in completed_signals {
        let signal = LuaSignal { id: sig_id };
        if let Err(e) = signal.fire(lua, ()) {
            println!("[TweenService] Error firing Completed signal: {}", e);
        }
    }
}

#[derive(Clone)]
pub struct LuaTween {
    pub id: u64,
    /// This token is delete automatically by Rust when Luau clean it (gc)
    pub _alive_token: Arc<()>,
}

impl UserData for LuaTween {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Completed", |lua, this| {
            if let Some(engine) = lua.app_data_ref::<TweenEngine>() {
                if let Some(tween) = engine.active_tweens.get(&this.id) {
                    return Ok(LuaSignal {
                        id: tween.completed_signal_id,
                    });
                }
            }
            Err(mlua::Error::runtime("Tween not found"))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Play", |lua, this, ()| {
            if let Some(mut engine) = lua.app_data_mut::<TweenEngine>() {
                if let Some(tween) = engine.active_tweens.get_mut(&this.id) {
                    if let Ok(instance) =
                        lua.registry_value::<mlua::AnyUserData>(&tween.instance_key)
                    {
                        for (prop_name, start_val, _end_val) in &mut tween.properties {
                            if let Ok(current_lua_val) =
                                instance.get::<mlua::Value>(prop_name.clone())
                            {
                                if let Some(current_val) = TweenableValue::from_lua(current_lua_val)
                                {
                                    *start_val = current_val;
                                }
                            }
                        }
                    }

                    if tween.state == PlayState::Completed {
                        tween.elapsed = 0.0;
                    }
                    tween.state = PlayState::Playing;
                } else {
                    println!(
                        "[TweenService] Error: Impossible to play the Tween {} (unfound)",
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
                                            (Some(s), Some(e)) => {
                                                let type_s = s.type_name();
                                                let type_e = e.type_name();
                                                if type_s == type_e {
                                                    props_to_animate.push((key, s, e));
                                                } else {
                                                    println!("[TweenService] Type error for '{}': expected '{}', receive '{}'", key, type_s, type_e);
                                                }
                                            }
                                            (None, _) => println!("[TweenService] Unsupported departure type '{}'", key),
                                            (_, None) => println!("[TweenService] Unsupported arrival type '{}'", key),
                                        }
                                    }
                                    Err(e) => println!("[TweenService] The property '{}' does not exist on this instance ! ({})", key, e),
                                }
                            }
                            Err(e) => println!("[TweenService] Error while processing on properties: {}", e),
                        }
                    }

                    if props_to_animate.is_empty() {
                        println!("[TweenService] WARN: Tween created without valid properties !");
                    }

                    let id = NEXT_TWEEN_ID.fetch_add(1, Ordering::Relaxed);
                    let instance_key = lua.create_registry_value(instance)?;
                    let completed_signal = LuaSignal::new(lua)?;

                    let alive_token = Arc::new(());
                    let lua_token = Arc::downgrade(&alive_token);

                    let active_tween = ActiveTween {
                        instance_key,
                        properties: props_to_animate,
                        info,
                        elapsed: 0.0,
                        state: PlayState::Paused,
                        completed_signal_id: completed_signal.id,
                        lua_token,
                    };

                    if let Some(mut engine) = lua.app_data_mut::<TweenEngine>() {
                        engine.active_tweens.insert(id, active_tween);
                    } else {
                        println!("[TweenService] ENGINE ISSUE: TweenEngine unfound in the memory !");
                    }

                    Ok(LuaTween { id, _alive_token: alive_token })
                },
            )?,
        )?;

        lua.globals().set("TweenService", ts)?;
        Ok(())
    }
}
