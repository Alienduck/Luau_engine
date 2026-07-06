use crate::types::{
    gui_object::GuiObject,
    instance::{CloneableInstance, InstanceData},
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

/// Luau-facing `Frame` — a rectangular 2-D UI element.
///
/// Position and size are expressed as [`LuaUDim2`] values (scale + offset),
/// following the Roblox convention.  The frame maps to a Bevy [`Node`] with
/// `PositionType::Absolute`.
#[derive(Clone)]
pub struct LuaFrame {
    pub base: InstanceData,
    pub gui: GuiObject,
}

impl CloneableInstance for LuaFrame {
    fn base(&self) -> &InstanceData {
        &self.base
    }

    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {
        // Layout is applied lazily via `update_layout` when properties are set.
    }
}

impl UserData for LuaFrame {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);
        crate::impl_gui_object_fields!(fields);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
    }
}

pub struct FrameModule;

impl LuaModule for FrameModule {
    fn name() -> &'static str {
        "Frame"
    }

    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                // TODO: replace with a command impl Scene with new bsn from
                // Bevy 0.19
                q.push_raw(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                        ))
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                });

                let frame = LuaFrame {
                    base: InstanceData::new(handle, q.clone(), "Frame"),
                    gui: GuiObject::default(),
                };
                let ud = lua_ctx.create_userdata(frame)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("Frame", t)
    }
}
