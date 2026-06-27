use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_rapier3d::{
    control::{CharacterLength, KinematicCharacterController},
    dynamics::RigidBody,
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
    controllers: Query<(&ChildOf, &LuauCharacterController)>,
    mut parents: Query<&mut KinematicCharacterController>,
) {
    let fixed_dt = 1.0 / 60.0;

    for (parent, ctrl) in controllers.iter() {
        let offset = ctrl.velocity * fixed_dt;

        if let Ok(mut kcc) = parents.get_mut(parent.get()) {
            kcc.translation = Some(offset);
        } else {
            commands.entity(parent.get()).insert((
                KinematicCharacterController {
                    translation: Some(offset),
                    snap_to_ground: Some(CharacterLength::Absolute(0.1)),
                    offset: CharacterLength::Absolute(0.01),
                    slide: true,
                    ..default()
                },
                RigidBody::KinematicPositionBased,
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
