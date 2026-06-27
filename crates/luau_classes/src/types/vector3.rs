use bevy::math::Vec3;
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::{FromLua, Lua, UserData, UserDataFields, UserDataMethods};

#[derive(Clone, Copy, Debug, Default)]
pub struct LuaVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FromLua for LuaVector3 {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            other => Err(mlua::Error::runtime(format!(
                "expected Vector3, got {}",
                other.type_name()
            ))),
        }
    }
}

impl From<LuaVector3> for Vec3 {
    fn from(value: LuaVector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl UserData for LuaVector3 {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.y));
        fields.add_field_method_get("Z", |_, this| Ok(this.z));
        fields.add_field_method_set("X", |_, this, v: f32| {
            this.x = v;
            Ok(())
        });
        fields.add_field_method_set("Y", |_, this, v: f32| {
            this.y = v;
            Ok(())
        });
        fields.add_field_method_set("Z", |_, this, v: f32| {
            this.z = v;
            Ok(())
        });
        fields.add_field_method_get("Unit", |_, this| {
            let normalized = Vec3 {
                x: this.x,
                y: this.y,
                z: this.z,
            }
            .normalize();
            Ok(LuaVector3 {
                x: normalized.x,
                y: normalized.y,
                z: normalized.z,
            })
        });
        fields.add_field_method_get("Magnitude", |_, this| {
            Ok(Vec3 {
                x: this.x,
                y: this.y,
                z: this.z,
            }
            .length())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
            Ok(format!("({}, {}, {})", this.x, this.y, this.z))
        });
        methods.add_meta_method(mlua::MetaMethod::Add, |_, this, rhs: LuaVector3| {
            Ok(LuaVector3 {
                x: this.x + rhs.x,
                y: this.y + rhs.y,
                z: this.z + rhs.z,
            })
        });
        methods.add_meta_method(mlua::MetaMethod::Sub, |_, this, rhs: LuaVector3| {
            Ok(LuaVector3 {
                x: this.x - rhs.x,
                y: this.y - rhs.y,
                z: this.z - rhs.z,
            })
        });
        methods.add_meta_method(mlua::MetaMethod::Mul, |_, this, scalar: f32| {
            Ok(LuaVector3 {
                x: this.x * scalar,
                y: this.y * scalar,
                z: this.z * scalar,
            })
        });
    }
}

pub struct Vector3Module;

impl LuaModule for Vector3Module {
    fn name() -> &'static str {
        "Vector3"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(|_, (x, y, z): (f32, f32, f32)| Ok(LuaVector3 { x, y, z }))?,
        )?;
        t.set("zero", LuaVector3::default())?;
        t.set(
            "one",
            LuaVector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        )?;
        lua.globals().set("Vector3", t)
    }
}
