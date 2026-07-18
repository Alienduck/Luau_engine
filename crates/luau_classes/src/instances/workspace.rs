use crate::types::{
    instance::{CloneableInstance, InstanceData},
    raycast::{RaycastParams, workspace_raycast},
    vector3::LuaVector3,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

/// Marker component — identifies the singleton workspace root entity.
#[derive(Component)]
pub struct WorkspaceRoot;

/// Luau-facing `workspace` singleton — the root of the 3-D scene hierarchy.
///
/// Cannot be parented or cloned (mirrors Roblox semantics).  All 3-D parts
/// must ultimately be descendants of the workspace to be rendered and to
/// participate in physics.
#[derive(Clone)]
pub struct LuaWorkspace {
    pub base: InstanceData,
}

impl CloneableInstance for LuaWorkspace {
    fn base(&self) -> &InstanceData {
        &self.base
    }

    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert((WorkspaceRoot, Visibility::Inherited));
        }
    }
}

impl UserData for LuaWorkspace {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_get("ClassName", |_, this| Ok(this.base.class_name));
        fields.add_field_method_get("Parent", |_, _| Ok(None::<mlua::AnyUserData>));

        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });
        fields.add_field_method_set("Parent", |_, _, _: Option<mlua::AnyUserData>| {
            Err(mlua::Error::runtime("Workspace cannot be parented"))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);

        methods.add_method("Clone", |_, _, ()| -> mlua::Result<()> {
            Err(mlua::Error::runtime("Workspace cannot be cloned"))
        });

        methods.add_method("Destroy", |_, _, ()| -> mlua::Result<()> {
            Err(mlua::Error::runtime("Workspace cannot be destroyed"))
        });

        methods.add_method(
                    "Raycast",
                    |lua, _, (origin, direction, params): (LuaVector3, LuaVector3, Option<RaycastParams>)| {
                        let world_ptr = *lua
                            .app_data_ref::<*mut World>()
                            .ok_or_else(|| mlua::Error::runtime("World_ptr missing"))?;
                        let world = unsafe { &mut *world_ptr };
                        let origin_vec = Vec3::new(origin.x, origin.y, origin.z);
                        let direction_vec = Vec3::new(direction.x, direction.y, direction.z);
                        workspace_raycast(lua, world, origin_vec, direction_vec, params)
                    },
                );

        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
    }
}

pub struct WorkspaceModule;

impl LuaModule for WorkspaceModule {
    fn name() -> &'static str {
        "Workspace"
    }

    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let handle = next_handle();
        let destroying_signal_id = crate::types::signal::LuaSignal::new(lua)?.id;
        let q = queue.clone();

        q.push_raw(move |w: &mut World| {
            let entity = w
                .spawn((
                    Transform::default(),
                    Visibility::Inherited,
                    WorkspaceRoot,
                    LuauHandle(handle),
                ))
                .id();
            w.resource_mut::<HandleMap>().insert(handle, entity, None);
        });

        let ws = LuaWorkspace {
            base: InstanceData::new(handle, queue.clone(), "Workspace", destroying_signal_id),
        };
        let ud = lua.create_userdata(ws)?;
        lua.named_registry_value::<mlua::Table>("__instance_cache")?
            .set(handle, ud.clone())?;
        lua.set_named_registry_value("__workspace_instance", ud.clone())?;
        lua.globals().set("workspace", ud)?;
        Ok(())
    }
}

/// Synchronises Rapier dormancy flags with Bevy visibility.
///
/// Entities that become invisible (e.g. unparented from the workspace) have
/// their collider and rigid body disabled so they don't affect simulation.
pub fn sync_dormancy_system(
    mut commands: Commands,
    query: Query<(Entity, &InheritedVisibility), (With<LuauHandle>, Changed<InheritedVisibility>)>,
) {
    for (entity, inherited_visibility) in query.iter() {
        let Ok(mut cmds) = commands.get_entity(entity) else {
            continue;
        };
        if inherited_visibility.get() {
            cmds.remove::<ColliderDisabled>()
                .remove::<RigidBodyDisabled>();
        } else {
            cmds.insert((ColliderDisabled, RigidBodyDisabled));
        }
    }
}
