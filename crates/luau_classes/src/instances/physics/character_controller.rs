use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_rapier3d::{
    control::{
        CharacterAutostep, CharacterLength, KinematicCharacterController,
        KinematicCharacterControllerOutput,
    },
    plugin::RapierConfiguration,
};
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
    rapier_config_query: Query<&RapierConfiguration>,
    mut controllers: Query<(&ChildOf, &mut LuauCharacterController)>,
    mut parents: Query<(
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        Option<&bevy_rapier3d::prelude::GravityScale>,
    )>,
) {
    let fixed_dt = 1.0 / 60.0;
    let base_gravity = if let Ok(rapier_config) = rapier_config_query.single() {
        rapier_config.gravity.y
    } else {
        -9.81
    };
    for (parent, mut ctrl) in controllers.iter_mut() {
        let mut current_v_velocity = ctrl.vertical_velocity;
        if let Ok((mut kcc, kcc_output, gravity_scale)) = parents.get_mut(parent.get()) {
            let is_grounded = kcc_output.map(|o| o.grounded).unwrap_or(false);
            let scale = gravity_scale.map(|g| g.0).unwrap_or(1.0);
            let applied_gravity = base_gravity * scale;
            if is_grounded {
                if ctrl.wants_to_jump {
                    current_v_velocity = ctrl.jump_power;
                    ctrl.wants_to_jump = false;
                } else if current_v_velocity < 0.0 {
                    current_v_velocity = 0.0;
                }
            } else {
                current_v_velocity += applied_gravity * fixed_dt;
            }
            ctrl.vertical_velocity = current_v_velocity;
            let mut final_velocity = ctrl.velocity;
            final_velocity.y = current_v_velocity;
            kcc.translation = Some(final_velocity * fixed_dt);
        } else {
            commands.entity(parent.get()).insert((
                KinematicCharacterController {
                    snap_to_ground: Some(CharacterLength::Absolute(0.1)),
                    offset: CharacterLength::Absolute(0.02),
                    slide: true,
                    autostep: Some(CharacterAutostep {
                        max_height: CharacterLength::Absolute(0.5),
                        min_width: CharacterLength::Absolute(0.2),
                        include_dynamic_bodies: true,
                    }),
                    filter_flags: bevy_rapier3d::pipeline::QueryFilterFlags::EXCLUDE_SENSORS,
                    ..default()
                },
                bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
            ));
        }
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
                let ctrl = LuaCharacterController {
                    base: InstanceData::new(handle, q.clone(), "CharacterController"),
                    move_direction: LuaVector3::default(),
                    walk_speed: 16.0,
                    jump: false,
                    jump_power: 24.0,
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
