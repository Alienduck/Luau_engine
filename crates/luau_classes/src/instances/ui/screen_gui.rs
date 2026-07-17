use crate::types::instance::{CloneableInstance, InstanceData};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

/// Luau-facing `ScreenGui` — a full-screen UI root node.
///
/// Acts as the top-level parent for [`LuaFrame`] and other GUI objects.
/// Maps to a Bevy UI node sized at 100% × 100%.
#[derive(Clone)]
pub struct LuaScreenGui {
    pub base: InstanceData,
}

impl CloneableInstance for LuaScreenGui {
    fn base(&self) -> &InstanceData {
        &self.base
    }

    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {
        // The node is already spawned by the module constructor; nothing extra
        // needed on clone since spawn_base_entity creates Transform/Visibility.
    }
}

impl UserData for LuaScreenGui {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_get("ClassName", |_, this| Ok(this.base.class_name));
        fields.add_field_method_get("Parent", |lua, this| {
            let Some(parent_handle) = this.base.parent_handle else {
                return Ok(None);
            };
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            Ok(cache.get::<Option<mlua::AnyUserData>>(parent_handle)?)
        });

        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });
        fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            this.base.set_parent(lua, parent);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
    }
}

pub struct ScreenGuiModule;

impl LuaModule for ScreenGuiModule {
    fn name() -> &'static str {
        "ScreenGui"
    }

    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let destroying_signal_id = crate::types::signal::LuaSignal::new(lua_ctx)?.id;
                q.push_raw(move |w: &mut World| {
                    let entity = w
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        })
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                });

                let sg = LuaScreenGui {
                    base: InstanceData::new(handle, q.clone(), "ScreenGui", destroying_signal_id),
                };
                let ud = lua_ctx.create_userdata(sg)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("ScreenGui", t)
    }
}
