use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::prelude::*;
use engine_core::components::LuauCharacterController;
use luau_runtime::bridge::{
    handle::{HandleMap, next_handle},
    queue::EngineQueue,
};
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
    pub jump_power: f32,
    pub jump: bool,
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
        fields.add_field_method_get("WalkSpeed", |_, this| Ok(this.walk_speed));
        fields.add_field_method_set("WalkSpeed", |_, this, v: f32| {
            this.walk_speed = v;
            this.base.queue.push(
                luau_runtime::bridge::queue::EngineCommand::SetCharacterWalkSpeed {
                    handle: this.base.handle,
                    walk_speed: v,
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
        fields.add_field_method_set("Parent", |_, this, v: Option<mlua::AnyUserData>| {
            let old_handle = this.base.parent_handle;
            let new_handle = v
                .as_ref()
                .and_then(|ud| crate::types::instance::instance_handle_from_any(ud));

            let walk_speed = this.walk_speed;
            let jump_height = this.jump_power;
            this.base.queue.push_raw(move |w: &mut World| {
                if let Some(old_handle) = old_handle {
                    if let Some(old_entity) = w.resource::<HandleMap>().get_entity(old_handle) {
                        if let Ok(mut e) = w.get_entity_mut(old_entity) {
                            e.remove::<(
                                TnuaController<CharacterControllerScheme>,
                                TnuaConfig<CharacterControllerScheme>,
                                TnuaAvian3dSensorShape,
                                LuauCharacterController,
                            )>();
                        }
                    }
                }
                if let Some(new_handle) = new_handle {
                    if let Some(new_entity) = w.resource::<HandleMap>().get_entity(new_handle) {
                        let collider = w
                            .get::<Collider>(new_entity)
                            .cloned()
                            .unwrap_or_else(|| Collider::capsule(0.5, 1.0));

                        let config_handle = w
                            .resource_mut::<Assets<CharacterControllerSchemeConfig>>()
                            .add(CharacterControllerSchemeConfig {
                                basis: TnuaBuiltinWalkConfig {
                                    speed: walk_speed,
                                    float_height: 1.0,
                                    ..default()
                                },
                                jumping: TnuaBuiltinJumpConfig {
                                    height: jump_height,
                                    ..default()
                                },
                            });

                        if let Ok(mut e) = w.get_entity_mut(new_entity) {
                            e.insert((
                                TnuaController::<CharacterControllerScheme>::default(),
                                TnuaConfig::<CharacterControllerScheme>(config_handle),
                                TnuaAvian3dSensorShape(collider),
                                LuauCharacterController::default(),
                                Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
                                Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
                                LockedAxes::ROTATION_LOCKED,
                            ));
                        }
                    }
                }
            });
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

pub fn apply_controls(
    mut tnua_query: Query<(
        &mut TnuaController<CharacterControllerScheme>,
        &mut LuauCharacterController,
    )>,
    inputs: ResMut<ButtonInput<KeyCode>>,
) {
    let Ok((mut ctrl, mut luau_ctrl)) = tnua_query.single_mut() else {
        return;
    };
    ctrl.initiate_action_feeding();

    let mut direction = Vec3::ZERO;
    let (forward, behind, left, right, jump) = luau_ctrl.custom_inputs_or_default();

    if inputs.pressed(forward) {
        direction += Vec3::NEG_Z;
    }
    if inputs.pressed(behind) {
        direction += Vec3::Z;
    }
    if inputs.pressed(left) {
        direction += Vec3::NEG_X;
    }
    if inputs.pressed(right) {
        direction += Vec3::X;
    }
    ctrl.basis = TnuaBuiltinWalk {
        desired_motion: direction.normalize_or_zero(),
        ..default()
    };
    if inputs.pressed(jump) || luau_ctrl.jump {
        ctrl.action(CharacterControllerScheme::Jumping(Default::default()));
        luau_ctrl.jump = false;
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
                    jump_power: 24.0,
                    jump: false,
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
