use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use luau_runtime::{
    bridge::{handle::HandleMap, queue::EngineQueue},
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields};

pub struct LuaRigidbody {
    pub queue: EngineQueue,
    pub handle_parent: Option<u64>,
}

impl UserData for LuaRigidbody {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_set("Parent", |_, this, parent: Option<mlua::AnyUserData>| {
            let new_handle = match parent {
                Some(p) => Some(p.borrow::<crate::instances::part::LuaPart>()?.0.handle),
                None => None,
            };
            let old_handle = this.handle_parent;
            this.handle_parent = new_handle;
            this.queue
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
            lua.create_function(move |_, ()| {
                Ok(LuaRigidbody {
                    queue: q.clone(),
                    handle_parent: None,
                })
            })?,
        )?;
        lua.globals().set("Rigidbody", t)
    }
}
