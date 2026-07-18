use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship, prelude::*};
use engine_core::components::LuauCharacterController;
use luau_runtime::bridge::{handle::next_handle, queue::EngineQueue};
use mlua::{Lua, UserData};

use crate::types::{
    instance::{CloneableInstance, InstanceData},
    vector3::LuaVector3,
};

#[derive(Clone)]
pub struct LuaCharacterController {
    pub base: InstanceData,
    pub move_direction: LuaVector3,
    pub walk_speed: f32,
    pub jump: bool,
    pub jump_power: f32,
    pub no_clip: bool,
}

impl CloneableInstance for LuaCharacterController {
    fn base(&self) -> &InstanceData {
        &self.base
    }

    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LuauCharacterController::default());
        }
    }
}

impl UserData for LuaCharacterController {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);
        fields.add_field_method_get("MoveDirection", |_, this| Ok(this.move_direction));
        fields.add_field_method_set("MoveDirection", |_, this, v: LuaVector3| {
            this.move_direction = v;
            let velocity: Vec3 = v.into();
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetCharacterMovement {
                    handle: this.base.handle,
                    movement: velocity * this.walk_speed,
                },
            );
            Ok(())
        });
        fields.add_field_method_get("WalkSpeed", |_, this| Ok(this.walk_speed));
        fields.add_field_method_set("WalkSpeed", |_, this, v: f32| {
            this.walk_speed = v;
            let velocity = Vec3::new(
                this.move_direction.x,
                this.move_direction.y,
                this.move_direction.z,
            ) * v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetCharacterMovement {
                    handle: this.base.handle,
                    movement: velocity,
                },
            );
            Ok(())
        });
        fields.add_field_method_get("Jump", |_, this| Ok(this.jump));
        fields.add_field_method_set("Jump", |_, this, v: bool| {
            this.jump = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetCharacterJump {
                    handle: this.base.handle,
                    jump: v,
                },
            );
            Ok(())
        });
        fields.add_field_method_get("JumpPower", |_, this| Ok(this.jump_power));
        fields.add_field_method_set("JumpPower", |_, this, v: f32| {
            this.jump_power = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetCharacterJumpPower {
                    handle: this.base.handle,
                    jump_power: v,
                },
            );
            Ok(())
        });
        fields.add_field_method_get("NoClip", |_, this| Ok(this.no_clip));
        fields.add_field_method_set("NoClip", |_, this, v: bool| {
            this.no_clip = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetCharacterNoClip {
                    handle: this.base.handle,
                    no_clip: v,
                },
            );
            Ok(())
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
            Ok(this.base.name.clone())
        });
    }
}

pub fn sync_character_controllers(
    mut commands: Commands,
    time: Res<Time>,
    gravity: Res<avian3d::prelude::Gravity>,
    spatial_query: avian3d::prelude::SpatialQuery,
    mut controllers: Query<(&ChildOf, &mut LuauCharacterController)>,
    mut parents: Query<(
        Entity,
        Option<&mut avian3d::prelude::LinearVelocity>,
        Option<&avian3d::prelude::GravityScale>,
        Option<&avian3d::prelude::CollisionLayers>,
        Option<&avian3d::prelude::Collider>,
        Option<&bevy::prelude::GlobalTransform>,
    )>,
) {
    let base_gravity = gravity.0.y;
    let dt = time.delta_secs();
    for (parent, mut ctrl) in controllers.iter_mut() {
        let parent_entity = parent.get();
        let Ok((_, vel_opt, gravity_scale, collision_layers, collider, transform)) =
            parents.get_mut(parent_entity)
        else {
            continue;
        };
        let Some(mut velocity) = vel_opt else {
            commands.entity(parent_entity).insert((
                avian3d::prelude::RigidBody::Dynamic,
                avian3d::prelude::LockedAxes::ROTATION_LOCKED,
                avian3d::prelude::LinearVelocity::ZERO,
                avian3d::prelude::Friction::new(0.0)
                    .with_combine_rule(avian3d::prelude::CoefficientCombine::Min),
                avian3d::prelude::GravityScale(0.0),
            ));
            continue;
        };
        let mut current_v_velocity = ctrl.vertical_velocity;
        let jump_requested = ctrl.wants_to_jump;
        ctrl.wants_to_jump = false;
        let scale = gravity_scale.map(|g| g.0).unwrap_or(1.0);
        let applied_gravity = base_gravity * scale;
        if ctrl.no_clip {
            current_v_velocity = 0.0;
            commands
                .entity(parent_entity)
                .insert(avian3d::prelude::Sensor);
            velocity.0 = ctrl.velocity;
        } else {
            commands
                .entity(parent_entity)
                .remove::<avian3d::prelude::Sensor>();
            let mut is_grounded = false;
            if let (Some(coll), Some(tf)) = (collider, transform) {
                let aabb = coll.aabb(bevy::math::Vec3::ZERO, bevy::math::Quat::IDENTITY);
                let mut filter =
                    avian3d::prelude::SpatialQueryFilter::from_excluded_entities([parent_entity]);
                if let Some(layers) = collision_layers {
                    filter = filter.with_mask(layers.filters);
                }
                if spatial_query
                    .cast_ray(
                        tf.translation(),
                        bevy::math::Dir3::NEG_Y,
                        aabb.max.y + 0.1,
                        true,
                        &filter,
                    )
                    .is_some()
                {
                    is_grounded = true;
                }
            }
            if is_grounded {
                if jump_requested {
                    current_v_velocity = ctrl.jump_power;
                } else if current_v_velocity < 0.0 {
                    current_v_velocity = -0.1;
                }
            } else {
                current_v_velocity += applied_gravity * dt;
                if current_v_velocity > 0.0 && velocity.y <= 0.001 {
                    current_v_velocity = 0.0;
                }
            }
            velocity.x = ctrl.velocity.x;
            velocity.z = ctrl.velocity.z;
            velocity.y = current_v_velocity;
        }
        ctrl.vertical_velocity = current_v_velocity;
    }
}

pub struct CharacterControllerModule;
impl luau_runtime::registry::LuaModule for CharacterControllerModule {
    fn name() -> &'static str {
        "CharacterController"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let destroying_signal_id = crate::types::signal::LuaSignal::new(lua_ctx)?.id;
                let ctrl = LuaCharacterController {
                    base: InstanceData::new(
                        handle,
                        q.clone(),
                        "CharacterController",
                        destroying_signal_id,
                    ),
                    move_direction: LuaVector3::default(),
                    walk_speed: 16.0,
                    jump: false,
                    jump_power: 24.0,
                    no_clip: false,
                };
                let c = ctrl.clone();
                q.push_raw(move |w: &mut World| {
                    let e = c.base().spawn_base_entity(w);
                    c.apply_bevy_components(e, w);
                });
                let ud = lua_ctx.create_userdata(ctrl)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("CharacterController", t)
    }
}
