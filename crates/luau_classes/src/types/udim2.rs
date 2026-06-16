use bevy::ui::Val;
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::{FromLua, Lua, UserData, UserDataFields};

#[derive(Clone, Copy, Debug, Default)]
pub struct LuaUDim2 {
    pub x_scale: f32,
    pub x_offset: f32,
    pub y_scale: f32,
    pub y_offset: f32,
}

impl LuaUDim2 {
    pub fn to_bevy_val_x(&self) -> Val {
        if self.x_scale > 0.0 {
            Val::Percent(self.x_scale * 100.0)
        } else {
            Val::Px(self.x_offset)
        }
    }
    pub fn to_bevy_val_y(&self) -> Val {
        if self.y_scale > 0.0 {
            Val::Percent(self.y_scale * 100.0)
        } else {
            Val::Px(self.y_offset)
        }
    }
}

impl FromLua for LuaUDim2 {
    fn from_lua(value: mlua::prelude::LuaValue, _: &Lua) -> mlua::prelude::LuaResult<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::runtime("expected Udim2")),
        }
    }
}

impl UserData for LuaUDim2 {
    fn add_fields<F: UserDataFields<Self>>(_fields: &mut F) {}
}

pub struct Udim2Module;

impl LuaModule for Udim2Module {
    fn name() -> &'static str {
        "UDim2"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(|_, (xs, ys, xo, yo): (f32, f32, f32, f32)| {
                Ok(LuaUDim2 {
                    x_scale: xs,
                    y_scale: ys,
                    x_offset: xo,
                    y_offset: yo,
                })
            })?,
        )?;
        t.set(
            "fromScale",
            lua.create_function(|_, (xs, ys): (f32, f32)| {
                Ok(LuaUDim2 {
                    x_scale: xs,
                    y_scale: ys,
                    x_offset: 0.,
                    y_offset: 0.,
                })
            })?,
        )?;
        t.set(
            "fromOffset",
            lua.create_function(|_, (xo, yo): (f32, f32)| {
                Ok(LuaUDim2 {
                    x_scale: 0.,
                    y_scale: 0.,
                    x_offset: xo,
                    y_offset: yo,
                })
            })?,
        )?;
        lua.globals().set("UDim2", t)
    }
}
