use crate::types::instance::InstanceData;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{ColliderDisabled, RigidBodyDisabled};
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields};

#[derive(Component)]
pub struct WorkspaceRoot;

pub struct LuaWorkspace {
    pub base: InstanceData,
}

impl UserData for LuaWorkspace {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });

        fields.add_field_method_get("Parent", |_, _| Ok(None::<mlua::AnyUserData>));
        fields.add_field_method_set("Parent", |_, _, _: Option<mlua::AnyUserData>| {
            Err(mlua::Error::runtime("Workspace cannot be parented"))
        });
    }
}

pub struct WorkspaceModule;

impl LuaModule for WorkspaceModule {
    fn name() -> &'static str {
        "Workspace"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let handle = next_handle();
        let q = queue.clone();

        q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
            let entity = w
                .spawn((Transform::default(), Visibility::Inherited, WorkspaceRoot))
                .id();
            w.resource_mut::<HandleMap>().insert(handle, entity, None);
        }));

        let ws = LuaWorkspace {
            base: InstanceData::new(handle, queue.clone(), "Workspace"),
        };
        let userdata = lua.create_userdata(ws)?;

        lua.set_named_registry_value("__workspace_instance", userdata.clone())?;
        lua.globals().set("workspace", userdata)?;
        Ok(())
    }
}

pub fn sync_dormancy_system(
    mut commands: Commands,
    query: Query<(Entity, &InheritedVisibility), (With<LuauHandle>, Changed<InheritedVisibility>)>,
) {
    for (entity, inherited_visibility) in query.iter() {
        if inherited_visibility.get() {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.remove::<ColliderDisabled>()
                    .remove::<RigidBodyDisabled>();
            }
        } else {
            if let Ok(mut cmds) = commands.get_entity(entity) {
                cmds.insert((ColliderDisabled, RigidBodyDisabled));
            }
        }
    }
}
