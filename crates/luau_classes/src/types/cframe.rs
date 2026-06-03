use crate::types::vector3::LuaVector3;
use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::{FromLua, Lua, UserData, UserDataFields, UserDataMethods};

/// CFrame = position (Vec3) + rotation (Quat).
/// Mirrors Roblox's CFrame API surface.
#[derive(Clone, Copy, Debug)]
pub struct LuaCFrame {
    pub position: bevy::math::Vec3,
    pub rotation: bevy::math::Quat,
}

impl Default for LuaCFrame {
    fn default() -> Self {
        Self {
            position: bevy::math::Vec3::ZERO,
            rotation: bevy::math::Quat::IDENTITY,
        }
    }
}

impl FromLua for LuaCFrame {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            other => Err(mlua::Error::runtime(format!(
                "expected CFrame, got {}",
                other.type_name()
            ))),
        }
    }
}

impl UserData for LuaCFrame {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        // Position vector
        fields.add_field_method_get("Position", |_, this| {
            Ok(LuaVector3 {
                x: this.position.x,
                y: this.position.y,
                z: this.position.z,
            })
        });
        // Orthonormal basis vectors (read-only, derived from rotation)
        fields.add_field_method_get("LookVector", |_, this| {
            let v = this.rotation * bevy::math::Vec3::NEG_Z;
            Ok(LuaVector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            })
        });
        fields.add_field_method_get("RightVector", |_, this| {
            let v = this.rotation * bevy::math::Vec3::X;
            Ok(LuaVector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            })
        });
        fields.add_field_method_get("UpVector", |_, this| {
            let v = this.rotation * bevy::math::Vec3::Y;
            Ok(LuaVector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            })
        });
        // Raw components
        fields.add_field_method_get("X", |_, this| Ok(this.position.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.position.y));
        fields.add_field_method_get("Z", |_, this| Ok(this.position.z));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
            Ok(format!(
                "CFrame({}, {}, {})",
                this.position.x, this.position.y, this.position.z
            ))
        });

        // CFrame * CFrame  — compose transforms
        methods.add_meta_method(mlua::MetaMethod::Mul, |_, this, rhs: LuaCFrame| {
            Ok(LuaCFrame {
                position: this.position + this.rotation * rhs.position,
                rotation: this.rotation * rhs.rotation,
            })
        });

        // Inverse
        methods.add_method("Inverse", |_, this, ()| {
            let inv_rot = this.rotation.inverse();
            Ok(LuaCFrame {
                position: inv_rot * (-this.position),
                rotation: inv_rot,
            })
        });

        // Lerp between two CFrames
        methods.add_method("Lerp", |_, this, (goal, t): (LuaCFrame, f32)| {
            Ok(LuaCFrame {
                position: this.position.lerp(goal.position, t),
                rotation: this.rotation.slerp(goal.rotation, t),
            })
        });

        // Convert to angle-axis Euler (YXZ, degrees) for convenience
        methods.add_method("ToEulerAnglesYXZ", |_, this, ()| {
            let (y, x, z) = this.rotation.to_euler(bevy::math::EulerRot::YXZ);
            Ok((x, y, z))
        });
    }
}

pub struct CFrameModule;

impl LuaModule for CFrameModule {
    fn name() -> &'static str {
        "CFrame"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let t = lua.create_table()?;

        // CFrame.new(x, y, z)  — identity rotation at position
        t.set(
            "new",
            lua.create_function(|_, (x, y, z): (f32, f32, f32)| {
                Ok(LuaCFrame {
                    position: bevy::math::Vec3::new(x, y, z),
                    rotation: bevy::math::Quat::IDENTITY,
                })
            })?,
        )?;

        // CFrame.lookAt(position: Vector3, target: Vector3)
        t.set(
            "lookAt",
            lua.create_function(|_, (pos, target): (LuaVector3, LuaVector3)| {
                let p = bevy::math::Vec3::new(pos.x, pos.y, pos.z);
                let tgt = bevy::math::Vec3::new(target.x, target.y, target.z);
                let dir = (tgt - p).normalize_or_zero();
                let rot = if dir.length_squared() < 1e-8 {
                    bevy::math::Quat::IDENTITY
                } else {
                    bevy::math::Quat::from_rotation_arc(bevy::math::Vec3::NEG_Z, dir)
                };
                Ok(LuaCFrame {
                    position: p,
                    rotation: rot,
                })
            })?,
        )?;

        // CFrame.fromEulerAnglesYXZ(rx, ry, rz) in radians
        t.set(
            "fromEulerAnglesYXZ",
            lua.create_function(|_, (rx, ry, rz): (f32, f32, f32)| {
                Ok(LuaCFrame {
                    position: bevy::math::Vec3::ZERO,
                    rotation: bevy::math::Quat::from_euler(bevy::math::EulerRot::YXZ, ry, rx, rz),
                })
            })?,
        )?;

        // CFrame.identity
        t.set("identity", LuaCFrame::default())?;

        lua.globals().set("CFrame", t)
    }
}
