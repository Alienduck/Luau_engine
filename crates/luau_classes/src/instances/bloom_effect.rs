use crate::types::instance::{CloneableInstance, InstanceData};
use bevy::prelude::*;
use engine_core::components::LuauBloom;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields};

#[derive(Clone)]
pub struct LuaBloomEffect {
    pub base: InstanceData,
    pub intensity: f32,
    pub size: f32,
    pub threshold: f32,
}

impl CloneableInstance for LuaBloomEffect {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LuauBloom {
                intensity: self.intensity,
                size: self.size,
                threshold: self.threshold,
            });
        }
    }
}

impl UserData for LuaBloomEffect {
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
        fields.add_field_method_get("Intensity", |_, this| Ok(this.intensity));
        fields.add_field_method_set("Intensity", |_, this, v: f32| {
            this.intensity = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetBloomIntensity {
                    handle: this.base.handle,
                    intensity: v,
                },
            );
            Ok(())
        });
        fields.add_field_method_get("Size", |_, this| Ok(this.size));
        fields.add_field_method_set("Size", |_, this, v: f32| {
            this.size = v;
            this.base
                .queue
                .push(luau_runtime::bridge::queue::EngineCommand::SetBloomSize {
                    handle: this.base.handle,
                    size: v,
                });
            Ok(())
        });
        fields.add_field_method_get("Threshold", |_, this| Ok(this.threshold));
        fields.add_field_method_set("Threshold", |_, this, v: f32| {
            this.threshold = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetBloomThreshold {
                    handle: this.base.handle,
                    threshold: v,
                },
            );
            Ok(())
        });
    }
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
    }
}

pub struct BloomEffectModule;

impl LuaModule for BloomEffectModule {
    fn name() -> &'static str {
        "BloomEffect"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_cache, ()| {
                let handle = next_handle();
                let effect = LuaBloomEffect {
                    base: InstanceData::new(handle, q.clone(), "BloomEffect"),
                    intensity: 1.0,
                    size: 24.0,
                    threshold: 2.0,
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
        lua.globals().set("BloomEffect", t)
    }
}
