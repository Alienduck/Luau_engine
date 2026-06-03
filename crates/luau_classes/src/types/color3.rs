use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::{FromLua, Lua, UserData, UserDataFields, UserDataMethods};

#[derive(Clone, Copy, Debug, Default)]
pub struct LuaColor3 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl FromLua for LuaColor3 {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            other => Err(mlua::Error::runtime(format!(
                "expected Color3, got {}",
                other.type_name()
            ))),
        }
    }
}

impl UserData for LuaColor3 {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("R", |_, this| Ok(this.r));
        fields.add_field_method_get("G", |_, this| Ok(this.g));
        fields.add_field_method_get("B", |_, this| Ok(this.b));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
            Ok(format!("({}, {}, {})", this.r, this.g, this.b))
        });
        methods.add_method("Lerp", |_, this, (other, t): (LuaColor3, f32)| {
            Ok(LuaColor3 {
                r: this.r + (other.r - this.r) * t,
                g: this.g + (other.g - this.g) * t,
                b: this.b + (other.b - this.b) * t,
            })
        });
    }
}

pub struct Color3Module;

impl LuaModule for Color3Module {
    fn name() -> &'static str {
        "Color3"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(|_, (r, g, b): (f32, f32, f32)| Ok(LuaColor3 { r, g, b }))?,
        )?;
        t.set(
            "fromRGB",
            lua.create_function(|_, (r, g, b): (u8, u8, u8)| {
                Ok(LuaColor3 {
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                })
            })?,
        )?;
        lua.globals().set("Color3", t)
    }
}
