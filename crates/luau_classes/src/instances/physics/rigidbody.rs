use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

use crate::types::instance::{CloneableInstance, InstanceData};

/// Luau-facing `Rigidbody` — attaches a [`RigidBody::Dynamic`] component to
/// its parent entity when parented and removes it when unparented.
///
/// Designed to be parented to a `Part`; the physics body is added/removed on
/// the parent entity, not on the Rigidbody's own entity.
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
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_get("ClassName", |_, this| Ok(this.base.class_name));
        fields.add_field_method_get("Parent", |lua, this| {
            let Some(parent_handle) = this.base.parent_handle else {
                return Ok(None);
            };
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            Ok(cache.get::<Option<mlua::AnyUserData>>(parent_handle)?)
        });

        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });
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
                    if let Some(old_h) = old_handle {
                        if let Some(e) = w.resource::<HandleMap>().get_entity(old_h) {
                            if let Ok(mut em) = w.get_entity_mut(e) {
                                em.remove::<RigidBody>();
                            }
                        }
                    }
                    if let Some(new_h) = new_handle {
                        if let Some(e) = w.resource::<HandleMap>().get_entity(new_h) {
                            if let Ok(mut em) = w.get_entity_mut(e) {
                                em.insert(RigidBody::Dynamic);
                            }
                        }
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
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
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let rb = LuaRigidbody {
                    base: InstanceData::new(handle, q.clone(), "Rigidbody"),
                };

                let spawn_copy = rb.clone();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = spawn_copy.base().spawn_base_entity(w);
                    spawn_copy.apply_bevy_components(entity, w);
                }));

                let ud = lua_ctx.create_userdata(rb)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("Rigidbody", t)
    }
}
