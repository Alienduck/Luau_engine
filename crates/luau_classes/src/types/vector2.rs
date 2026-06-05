use luau_runtime::registry::LuaModule;
use mlua::{FromLua, MetaMethod::ToString, UserData};

#[derive(Clone, Copy, Debug, Default)]
pub struct LuaVector2 {
    pub x: f32,
    pub y: f32,
}

impl FromLua for LuaVector2 {
    fn from_lua(
        value: mlua::prelude::LuaValue,
        _lua: &mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        match value {
            mlua::Value::UserData(v) => Ok(*v.borrow::<Self>()?),
            other => Err(mlua::Error::runtime(format!(
                "expected Vector2 got {}",
                other.type_name()
            ))),
        }
    }
}

impl UserData for LuaVector2 {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.y));

        fields.add_field_method_set("X", |_, this, v: f32| {
            this.x = v;
            Ok(())
        });
        fields.add_field_method_set("Y", |_, this, v: f32| {
            this.y = v;
            Ok(())
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(ToString, |_, this, ()| {
            Ok(format!("({} {})", this.x, this.y))
        });
        methods.add_meta_method(mlua::MetaMethod::Add, |_, this, v: LuaVector2| {
            Ok(LuaVector2 {
                x: this.x + v.x,
                y: this.y + v.y,
            })
        });
        methods.add_meta_method(mlua::MetaMethod::Sub, |_, this, v: LuaVector2| {
            Ok(LuaVector2 {
                x: this.x - v.x,
                y: this.y - v.y,
            })
        });
        methods.add_meta_method(mlua::MetaMethod::Mul, |_, this, scalar: f32| {
            Ok(LuaVector2 {
                x: this.x * scalar,
                y: this.y * scalar,
            })
        });
    }
}

pub struct Vector2Module;

impl LuaModule for Vector2Module {
    fn name() -> &'static str {
        "Vector2"
    }

    fn register(
        lua: &mlua::prelude::Lua,
        _queue: &luau_runtime::bridge::queue::EngineQueue,
    ) -> mlua::Result<()> {
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(|_, (x, y): (f32, f32)| Ok(LuaVector2 { x, y }))?,
        )?;
        t.set(
            "zero",
            lua.create_function(|_, ()| Ok(LuaVector2::default()))?,
        )?;
        t.set(
            "one",
            lua.create_function(|_, ()| Ok(LuaVector2 { x: 1., y: 1. }))?,
        )?;
        lua.globals().set("Vector2", t)
    }
}
