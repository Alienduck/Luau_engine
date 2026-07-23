use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::prelude::*;
use engine_core::definitions::physics::character_controller::*;
use luau_runtime::bridge::handle::{HandleMap, next_handle};
use luau_runtime::bridge::queue::*;
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
    pub hip_height: f32,
}

impl LuaCharacterController {
    fn push_stat(&self, stat: CharacterStat) {
        if let Some(parent_handle) = self.base.parent_handle {
            self.base.queue.push(EngineCommand::SetCharacterStat {
                handle: parent_handle,
                stat,
            });
        }
    }
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
            this.push_stat(CharacterStat::WalkSpeed(v));
            Ok(())
        });
        fields.add_field_method_get("Jump", |_, this| Ok(this.jump));
        fields.add_field_method_set("Jump", |_, this, v: bool| {
            this.jump = v;
            this.push_stat(CharacterStat::Jump(v));
            Ok(())
        });
        fields.add_field_method_get("JumpPower", |_, this| Ok(this.jump_power));
        fields.add_field_method_set("JumpPower", |_, this, v: f32| {
            this.jump_power = v;
            this.push_stat(CharacterStat::JumpPower(v));
            Ok(())
        });
        fields.add_field_method_get("HipHeight", |_, this| Ok(this.hip_height));
        fields.add_field_method_set("HipHeight", |_, this, v: f32| {
            this.hip_height = v;
            this.push_stat(CharacterStat::HipHeight(v));
            Ok(())
        });
        fields.add_field_method_set("Parent", |_, this, v: Option<mlua::AnyUserData>| {
            let old_handle = this.base.parent_handle;
            let new_handle = v
                .as_ref()
                .and_then(|ud| crate::types::instance::instance_handle_from_any(ud));
            this.base.parent_handle = new_handle;

            let walk_speed = this.walk_speed;
            let jump_height = this.jump_power;
            let hip_height = this.hip_height;
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
                            .unwrap_or_else(|| Collider::capsule(1.0, 1.0));

                        let config_handle = w
                            .resource_mut::<Assets<CharacterControllerSchemeConfig>>()
                            .add(CharacterControllerSchemeConfig {
                                basis: TnuaBuiltinWalkConfig {
                                    speed: walk_speed,
                                    float_height: hip_height,
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
        if luau_ctrl.jump {
            luau_ctrl.jump = false;
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
                    jump_power: 24.0,
                    jump: false,
                    hip_height: 0.1,
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
