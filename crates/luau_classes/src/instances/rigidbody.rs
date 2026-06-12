use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields};

use crate::{
    impl_lua_clone,
    types::instance::{CloneableInstance, InstanceData},
};

#[derive(Clone)]
pub struct LuaRigidbody {
    pub base: InstanceData,
}

impl CloneableInstance for LuaRigidbody {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {}
}

impl UserData for LuaRigidbody {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            let old_handle = this.base.parent_handle;
            let new_handle = parent
                .as_ref()
                .and_then(|ud| crate::types::instance::instance_handle_from_any(ud));

            this.base.set_parent(lua, parent);
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(old) = old_handle {
                        if let Some(old_entity) = w.resource::<HandleMap>().get_entity(old) {
                            if let Ok(mut e) = w.get_entity_mut(old_entity) {
                                e.remove::<RigidBody>();
                            }
                        }
                    }

                    if let Some(new_h) = new_handle {
                        if let Some(new_entity) = w.resource::<HandleMap>().get_entity(new_h) {
                            if let Ok(mut e) = w.get_entity_mut(new_entity) {
                                e.insert(RigidBody::Dynamic);
                            }
                        }
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        impl_lua_clone!(methods);
    }
}

pub struct RigidbodyModule;

impl LuaModule for RigidbodyModule {
    fn name() -> &'static str {
        "Rigidbody"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_cache, ()| {
                let handle = next_handle();
                let rb = LuaRigidbody {
                    base: InstanceData::new(handle, q.clone(), "Rigidbody"),
                };

                let clone_for_spawn = rb.clone();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = clone_for_spawn.base().spawn_base_entity(w);
                    clone_for_spawn.apply_bevy_components(entity, w);
                }));

                let userdata = lua_cache.create_userdata(rb)?;

                if let Ok(cache) = lua_cache.named_registry_value::<mlua::Table>("__instance_cache")
                {
                    let _ = cache.set(handle, userdata.clone());
                }

                Ok(userdata)
            })?,
        )?;
        lua.globals().set("Rigidbody", t)
    }
}
