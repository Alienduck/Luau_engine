use crate::types::{instance::InstanceData, vector3::LuaVector3};
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

#[derive(Clone)]
pub struct LuaCollider {
    pub base: InstanceData,
    pub size: LuaVector3,
}

impl UserData for LuaCollider {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Size", |_, this| Ok(this.size));

        fields.add_field_method_set("Size", |_, this, v: LuaVector3| {
            this.size = v;
            let current_parent = this.base.parent_handle;
            let hx = v.x / 2.0;
            let hy = v.y / 2.0;
            let hz = v.z / 2.0;

            if let Some(handle) = current_parent {
                this.base
                    .queue
                    .0
                    .lock()
                    .unwrap()
                    .push(Box::new(move |w: &mut World| {
                        if let Some(entity) = w.resource::<HandleMap>().get_entity(handle) {
                            if let Ok(mut e) = w.get_entity_mut(entity) {
                                e.insert((
                                    Collider::cuboid(hx, hy, hz),
                                    ActiveEvents::COLLISION_EVENTS,
                                ));
                            }
                        }
                    }));
            }
            Ok(())
        });

        fields.add_field_method_set("Parent", |_, this, parent: Option<mlua::AnyUserData>| {
            let old_handle = this.base.parent_handle;
            let new_handle = parent
                .as_ref()
                .and_then(|ud| crate::types::instance::instance_handle_from_any(ud));

            this.base.set_parent(parent);

            let hx = this.size.x / 2.0;
            let hy = this.size.y / 2.0;
            let hz = this.size.z / 2.0;

            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(old) = old_handle {
                        if let Some(old_entity) = w.resource::<HandleMap>().get_entity(old) {
                            if let Ok(mut e) = w.get_entity_mut(old_entity) {
                                e.remove::<(Collider, ActiveEvents)>();
                            }
                        }
                    }

                    if let Some(new_h) = new_handle {
                        if let Some(new_entity) = w.resource::<HandleMap>().get_entity(new_h) {
                            if let Ok(mut e) = w.get_entity_mut(new_entity) {
                                e.insert((
                                    Collider::cuboid(hx, hy, hz),
                                    ActiveEvents::COLLISION_EVENTS,
                                ));
                            }
                        }
                    }
                }));
            Ok(())
        });
    }
}

pub struct ColliderModule;

impl LuaModule for ColliderModule {
    fn name() -> &'static str {
        "Collider"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |_, ()| {
                let handle = next_handle();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = w.spawn((Transform::default(),)).id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                }));

                Ok(LuaCollider {
                    base: InstanceData::new(handle, q.clone(), "Collider"),
                    size: LuaVector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                })
            })?,
        )?;
        lua.globals().set("Collider", t)
    }
}
