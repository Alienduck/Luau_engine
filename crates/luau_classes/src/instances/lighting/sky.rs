use crate::types::instance::{CloneableInstance, InstanceData};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};

#[derive(Component, Clone)]
pub struct LuauSky {
    pub cubemap_path: String,
}

#[derive(Clone)]
pub struct LuaSky {
    pub base: InstanceData,
    pub cubemap_path: String,
}

impl CloneableInstance for LuaSky {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LuauSky {
                cubemap_path: self.cubemap_path.clone(),
            });
        }
    }
}

impl UserData for LuaSky {
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
        fields.add_field_method_get("CubemapPath", |_, this| Ok(this.cubemap_path.clone()));
        fields.add_field_method_set("CubemapPath", |_, this, v: String| {
            this.cubemap_path = v.clone();
            let h = this.base.handle;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut sky) = w.get_mut::<LuauSky>(e) {
                            sky.cubemap_path = v;
                        }
                    }
                }));
            Ok(())
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
    }
}

pub struct SkyModule;
impl LuaModule for SkyModule {
    fn name() -> &'static str {
        "Sky"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_cache, ()| {
                let handle = next_handle();
                let effect = LuaSky {
                    base: InstanceData::new(handle, q.clone(), "Sky"),
                    cubemap_path: "".to_string(),
                };
                let clone_for_spawn = effect.clone();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = clone_for_spawn.base().spawn_base_entity(w);
                    clone_for_spawn.apply_bevy_components(entity, w);
                }));
                let userdata = lua_cache.create_userdata(effect)?;
                if let Ok(cache) = lua_cache.named_registry_value::<mlua::Table>("__instance_cache")
                {
                    let _ = cache.set(handle, userdata.clone());
                }
                Ok(userdata)
            })?,
        )?;
        lua.globals().set("Sky", t)
    }
}
