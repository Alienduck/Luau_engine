use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::prelude::*;
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

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum CharacterControllerScheme {
    Jumping(TnuaBuiltinJump),
}

pub fn sync_character_controllers(
    mut commands: Commands,
    mut controllers: Query<(&ChildOf, &mut LuauCharacterController)>,
    mut parents: Query<(
        Entity,
        Option<&mut TnuaController<CharacterControllerScheme>>,
        Option<&mut LinearVelocity>,
    )>,
    mut control_scheme_config: ResMut<Assets<CharacterControllerSchemeConfig>>,
) {
    for (parent, mut ctrl) in controllers.iter_mut() {
        let parent_entity = parent.get();
        let Ok((_, tnua_opt, vel_opt)) = parents.get_mut(parent_entity) else {
            continue;
        };
        if tnua_opt.is_none() {
            commands.entity(parent_entity).insert((
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                LinearVelocity::ZERO,
                Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
                GravityScale(1.0),
                MassPropertiesBundle::from_shape(&Collider::sphere(0.5), 1.0),
                TnuaController::<CharacterControllerScheme>::default(),
                TnuaConfig::<CharacterControllerScheme>(control_scheme_config.add(
                    CharacterControllerSchemeConfig {
                        basis: TnuaBuiltinWalkConfig {
                            speed: ctrl.walk_speed,
                            ..default()
                        },
                        jumping: TnuaBuiltinJumpConfig {
                            height: ctrl.jump_power,
                            ..default()
                        },
                    },
                )),
                TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
            ));
            continue;
        }
        let mut controller = tnua_opt.unwrap();

        controller.initiate_action_feeding();

        let jump_requested = ctrl.wants_to_jump;
        ctrl.wants_to_jump = false;

        if ctrl.no_clip {
            commands.entity(parent_entity).insert(Sensor);
            if let Some(mut vel) = vel_opt {
                vel.0 = ctrl.velocity;
            }
        } else {
            commands.entity(parent_entity).remove::<Sensor>();

            let desired_velocity = Vec3::new(ctrl.velocity.x, 0.0, ctrl.velocity.z);

            controller.basis = TnuaBuiltinWalk {
                desired_motion: desired_velocity,
                ..default()
            };

            if jump_requested {
                controller.action(CharacterControllerScheme::Jumping(TnuaBuiltinJump {
                    horizontal_displacement: Some(Vec3::new(0.0, ctrl.jump_power * 0.1, 0.0)),
                    ..default()
                }));
            }
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
