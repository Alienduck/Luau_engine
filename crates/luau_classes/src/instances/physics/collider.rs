use crate::{
    instances::mesh_part::LuauCollisionFidelity,
    types::{
        enums::LuauPartShape,
        instance::{CloneableInstance, InstanceData},
        signal::LuaSignal,
        vector3::LuaVector3,
    },
};
use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship, prelude::*};
use engine_core::definitions::physics::collision_group::PhysicsCollisionGroups;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

/// Luau-facing `Collider` — attaches a box [`Collider`] component to its
/// parent entity when parented and removes it when unparented.
///
/// Like [`LuaRigidbody`], the collider is added/removed on the *parent*
/// entity so that a single `Part` can own both.
#[derive(Clone)]
pub struct LuaCollider {
    pub base: InstanceData,
    /// Half-extents of the box collider, expressed as full size (divided by 2
    /// when passed to Rapier).
    pub size: LuaVector3,
    pub friction: f32,
    pub can_collide: bool,
    pub restitution: f32,
    pub collision_group: String,
}

impl CloneableInstance for LuaCollider {
    fn base(&self) -> &InstanceData {
        &self.base
    }

    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {}
}

impl UserData for LuaCollider {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);
        fields.add_field_method_get("Size", |_, this| Ok(this.size));
        fields.add_field_method_set("Size", |_, this, v: LuaVector3| {
            this.size = v;
            Ok(())
        });
        fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            let old_handle = this.base.parent_handle;
            let new_handle = parent
                .as_ref()
                .and_then(|ud| crate::types::instance::instance_handle_from_any(ud));
            let (hx, hy, hz) = (this.size.x / 2.0, this.size.y / 2.0, this.size.z / 2.0);

            let collision_group = this.collision_group.clone();

            this.base.set_parent(lua, parent);

            let friction = this.friction;
            let restitution = this.restitution;
            let can_collide = this.can_collide;

            this.base.queue.push_raw(move |w: &mut World| {
                if let Some(old_h) = old_handle {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(old_h) {
                        if let Ok(mut em) = w.get_entity_mut(e) {
                            em.remove::<(
                                Collider,
                                CollisionEventsEnabled,
                                ColliderConstructorHierarchy,
                                SweptCcd,
                                Sensor,
                                Friction,
                                Restitution,
                                CollisionLayers,
                            )>();
                        }
                    }
                }
                if let Some(new_h) = new_handle {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(new_h) {
                        if let Ok(mut em) = w.get_entity_mut(e) {
                            em.insert((
                                SweptCcd::default(),
                                CollisionEventsEnabled,
                                Friction::new(friction),
                                Restitution::new(restitution),
                            ));
                            if !can_collide {
                                em.insert(Sensor);
                            } else {
                                em.remove::<Sensor>();
                            }
                            if let Some(fidelity) = em.get::<LuauCollisionFidelity>() {
                                match fidelity {
                                    LuauCollisionFidelity::Hull => {
                                        em.insert(ColliderConstructorHierarchy {
                                            default_constructor: Some(
                                                ColliderConstructor::ConvexHullFromMesh,
                                            ),
                                            ..default()
                                        });
                                    }
                                    LuauCollisionFidelity::Precise => {
                                        em.insert(ColliderConstructorHierarchy {
                                            default_constructor: Some(
                                                ColliderConstructor::TrimeshFromMeshWithConfig(
                                                    TrimeshFlags::all(),
                                                ),
                                            ),
                                            ..default()
                                        });
                                    }
                                    LuauCollisionFidelity::Default => {
                                        em.insert(ColliderConstructorHierarchy {
                                            default_constructor: Some(
                                                ColliderConstructor::TrimeshFromMesh,
                                            ),
                                            ..default()
                                        });
                                    }
                                    LuauCollisionFidelity::Box => {
                                        em.insert(Collider::cuboid(hx, hy, hz));
                                    }
                                }
                            } else if let Some(shape) = em.get::<LuauPartShape>() {
                                em.insert(match shape {
                                    LuauPartShape::Ball => Collider::sphere(hx.max(hy).max(hz)),
                                    LuauPartShape::Cylinder => Collider::cylinder(hy, hx.max(hz)),
                                    LuauPartShape::Block => Collider::cuboid(hx, hy, hz),
                                    LuauPartShape::Capsule => Collider::capsule(hy, hx.max(hz)),
                                });
                            } else {
                                em.insert(Collider::cuboid(hx, hy, hz));
                            }
                        }
                    }
                }
            });
            if let Some(new_h) = new_handle {
                this.base.queue.push(
                    luau_runtime::bridge::queue::EngineCommand::SetCollisionGroup {
                        handle: new_h,
                        group: collision_group,
                    },
                );
            }
            Ok(())
        });
        fields.add_field_method_get("CanCollide", |_, this| Ok(this.can_collide));
        fields.add_field_method_set("CanCollide", |_, this, v: bool| {
            this.can_collide = v;
            if let Some(parent_h) = this.base.parent_handle {
                this.base.queue.push(
                    luau_runtime::bridge::queue::EngineCommand::SetColliderCollide {
                        handle: parent_h,
                        can_collide: v,
                    },
                );
            }
            Ok(())
        });
        fields.add_field_method_get("Friction", |_, this| Ok(this.friction));
        fields.add_field_method_set("Friction", |_, this, v: f32| {
            this.friction = v;
            if let Some(parent_h) = this.base.parent_handle {
                this.base.queue.push(
                    luau_runtime::bridge::queue::EngineCommand::SetColliderFriction {
                        handle: parent_h,
                        friction: v,
                    },
                );
            }
            Ok(())
        });
        fields.add_field_method_get("Restitution", |_, this| Ok(this.restitution));
        fields.add_field_method_set("Restitution", |_, this, v: f32| {
            this.restitution = v;
            if let Some(parent_h) = this.base.parent_handle {
                this.base.queue.push(
                    luau_runtime::bridge::queue::EngineCommand::SetColliderRestitution {
                        handle: parent_h,
                        restitution: v,
                    },
                );
            }
            Ok(())
        });
        fields.add_field_method_get("CollisionGroup", |_, this| Ok(this.collision_group.clone()));
        fields.add_field_method_set("CollisionGroup", |_, this, v: String| {
            this.collision_group = v.clone();
            if let Some(parent_h) = this.base.parent_handle {
                this.base.queue.push(
                    luau_runtime::bridge::queue::EngineCommand::SetCollisionGroup {
                        handle: parent_h,
                        group: v,
                    },
                );
            }
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
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
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let destroy_signal_id = LuaSignal::new(lua_ctx)?;
                let col = LuaCollider {
                    base: InstanceData::new(handle, q.clone(), "Collider", destroy_signal_id.id),
                    size: LuaVector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    friction: 0.0,
                    restitution: 0.0,
                    can_collide: true,
                    collision_group: "Default".into(),
                };
                let spawn_copy = col.clone();
                q.push_raw(move |w: &mut World| {
                    let entity = spawn_copy.base().spawn_base_entity(w);
                    spawn_copy.apply_bevy_components(entity, w);
                });
                let ud = lua_ctx.create_userdata(col)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("Collider", t)
    }
}

pub fn apply_default_collision_groups_system(
    mut commands: Commands,
    query: Query<(Entity, Option<&ChildOf>), (With<Collider>, Without<CollisionLayers>)>,
    parent_query: Query<&ChildOf>,
    group_query: Query<&CollisionLayers>,
    registry: Res<PhysicsCollisionGroups>,
) {
    if query.is_empty() {
        return;
    }

    let default_layer = registry.masks[0];
    let mut members = LayerMask::NONE;
    members.add(1);
    let mut filters = LayerMask::NONE;
    filters.add(default_layer);
    let default_groups = CollisionLayers::new(members, default_layer);

    for (entity, opt_parent) in query.iter() {
        let mut applied_groups = default_groups;

        let mut current_opt = opt_parent;
        while let Some(parent) = current_opt {
            if let Ok(parent_groups) = group_query.get(parent.get()) {
                applied_groups = *parent_groups;
                break;
            }
            current_opt = parent_query.get(parent.get()).ok();
        }

        commands.entity(entity).insert(applied_groups);
    }
}
