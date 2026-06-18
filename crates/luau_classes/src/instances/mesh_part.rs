use crate::{
    impl_base_instance_fields,
    instances::base_part::BasePartData,
    types::{
        instance::{CloneableInstance, InstanceData},
        signal::LuaSignal,
        vector3::LuaVector3,
    },
};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

#[derive(Clone)]
pub struct LuaMeshPart {
    pub base_part_data: BasePartData,
    pub mesh_id: String,
}

impl CloneableInstance for LuaMeshPart {
    fn base(&self) -> &InstanceData {
        &self.base_part_data.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base_part_data.base
    }
    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {}
}

impl UserData for LuaMeshPart {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        impl_base_instance_fields!(fields);

        fields.add_field_method_get("MeshId", |_, this| Ok(this.mesh_id.clone()));
        fields.add_field_method_set("MeshId", |_, this, v: String| {
            this.mesh_id = v.clone();
            let h = this.base_part_data.base.handle;
            this.base_part_data
                .base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let handle: Handle<Scene> = w.resource::<AssetServer>().load(&v);
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        w.entity_mut(e).insert(SceneRoot(handle));
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Position", |_, this| {
            let p = this.base_part_data.cframe.position;
            Ok(LuaVector3 {
                x: p.x,
                y: p.y,
                z: p.z,
            })
        });
        fields.add_field_method_set("Position", |_, this, v: LuaVector3| {
            this.base_part_data.set_position(v);
            Ok(())
        });

        fields.add_field_method_get("Size", |_, this| Ok(this.base_part_data.size));
        fields.add_field_method_set("Size", |_, this, v: LuaVector3| {
            this.base_part_data.size = v;
            let h = this.base_part_data.base.handle;
            this.base_part_data
                .base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut t) = w.get_mut::<Transform>(e) {
                            t.scale = Vec3::new(v.x, v.y, v.z);
                        }
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base().name.clone()));
    }
}

pub struct MeshPartModule;

impl LuaModule for MeshPartModule {
    fn name() -> &'static str {
        "MeshPart"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let touch_signal_id = LuaSignal::new(lua_ctx)?;
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Transform::default(),
                            Visibility::default(),
                            luau_runtime::bridge::handle::LuauHandle(handle),
                        ))
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                }));

                let base_part_data = BasePartData::new(handle, q.clone(), touch_signal_id.id);

                let mesh_part = LuaMeshPart {
                    base_part_data,
                    mesh_id: "".to_string(),
                };

                let ud = lua_ctx.create_userdata(mesh_part)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("MeshPart", t)
    }
}
