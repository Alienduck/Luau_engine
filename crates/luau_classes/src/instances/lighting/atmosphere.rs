use crate::types::{
    color3::LuaColor3,
    instance::{CloneableInstance, InstanceData},
};
use bevy::prelude::*;
use engine_core::components::LuauAtmosphere;
use luau_runtime::{
    bridge::{handle::next_handle, queue::EngineQueue},
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};

#[derive(Clone)]
pub struct LuaAtmosphere {
    pub base: InstanceData,
    pub density: f32,
    pub color: Color,
    pub decay: Color,
    pub glare: f32,
    pub haze: f32,
}

impl CloneableInstance for LuaAtmosphere {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LuauAtmosphere {
                density: self.density,
                color: self.color,
                decay: self.decay,
                glare: self.glare,
                haze: self.haze,
            });
        }
    }
}

impl UserData for LuaAtmosphere {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });

        fields.add_field_method_get("Parent", |lua, this| {
            if let Some(parent_handle) = this.base.parent_handle {
                let instances_map: mlua::Table = lua.named_registry_value("__instance_cache")?;
                Ok(instances_map.get::<mlua::AnyUserData>(parent_handle).ok())
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            this.base.set_parent(lua, parent);
            Ok(())
        });

        fields.add_field_method_get("Density", |_, this| Ok(this.density));
        fields.add_field_method_set("Density", |_, this, v: f32| {
            this.density = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetAtmosphereDensity {
                    handle: this.base.handle,
                    density: v,
                },
            );
            Ok(())
        });

        fields.add_field_method_get("Color", |_, this| Ok(LuaColor3::from(this.color)));
        fields.add_field_method_set("Color", |_, this, v: LuaColor3| {
            let color = Color::srgb(v.r, v.g, v.b);
            this.color = color;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetAtmosphereColor {
                    handle: this.base.handle,
                    color,
                },
            );
            Ok(())
        });

        fields.add_field_method_get("Decay", |_, this| Ok(LuaColor3::from(this.decay)));
        fields.add_field_method_set("Decay", |_, this, v: LuaColor3| {
            let decay = Color::srgb(v.r, v.g, v.b);
            this.decay = decay;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetAtmosphereDecay {
                    handle: this.base.handle,
                    decay,
                },
            );
            Ok(())
        });

        fields.add_field_method_get("Glare", |_, this| Ok(this.glare));
        fields.add_field_method_set("Glare", |_, this, v: f32| {
            this.glare = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetAtmosphereGlare {
                    handle: this.base.handle,
                    glare: v,
                },
            );
            Ok(())
        });

        fields.add_field_method_get("Haze", |_, this| Ok(this.haze));
        fields.add_field_method_set("Haze", |_, this, v: f32| {
            this.haze = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetAtmosphereHaze {
                    handle: this.base.handle,
                    haze: v,
                },
            );
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
    }
}

pub struct AtmosphereModule;

impl LuaModule for AtmosphereModule {
    fn name() -> &'static str {
        "Atmosphere"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_cache, ()| {
                let handle = next_handle();
                let effect = LuaAtmosphere {
                    base: InstanceData::new(handle, q.clone(), "Atmosphere"),
                    density: 0.3,
                    color: Color::srgb(0.78, 0.78, 0.78),
                    decay: Color::srgb(0.41, 0.44, 0.49),
                    glare: 0.0,
                    haze: 0.0,
                };

                let clone_for_spawn = effect.clone();
                q.push_raw(move |w: &mut World| {
                    let entity = clone_for_spawn.base().spawn_base_entity(w);
                    clone_for_spawn.apply_bevy_components(entity, w);
                });

                let userdata = lua_cache.create_userdata(effect)?;
                if let Ok(cache) = lua_cache.named_registry_value::<mlua::Table>("__instance_cache")
                {
                    let _ = cache.set(handle, userdata.clone());
                }
                Ok(userdata)
            })?,
        )?;
        lua.globals().set("Atmosphere", t)
    }
}
