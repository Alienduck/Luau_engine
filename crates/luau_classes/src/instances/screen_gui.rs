use crate::types::instance::InstanceData;
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData};

pub struct LuaScreenGui {
    pub base: InstanceData,
}

impl UserData for LuaScreenGui {}

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
            lua.create_function(move |_, ()| {
                let handle = next_handle();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = w
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },))
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                }));
                Ok(LuaScreenGui {
                    base: InstanceData::new(handle, q.clone(), "ScreenGui"),
                })
            })?,
        )?;
        lua.globals().set("ScreenGui", t)
    }
}
