use crate::types::{
    color3::LuaColor3,
    tween_info::{EasingDirection, EasingStyle},
    udim2::LuaUDim2,
    vector2::LuaVector2,
    vector3::LuaVector3,
};
use mlua::Lua;

#[derive(Clone, Debug)]
pub enum TweenableValue {
    Float(f32),
    Color3(LuaColor3),
    UDim2(LuaUDim2),
    Vector2(LuaVector2),
    Vector3(LuaVector3),
}

impl TweenableValue {
    /// Try to convert a Lua dynamic value in simple Rust math value
    pub fn from_lua(value: mlua::Value) -> Option<Self> {
        match value {
            mlua::Value::Number(n) => Some(Self::Float(n as f32)),
            mlua::Value::Integer(i) => Some(Self::Float(i as f32)),
            mlua::Value::UserData(ud) => {
                if let Ok(c) = ud.borrow::<LuaColor3>() {
                    return Some(Self::Color3(c.clone()));
                }
                if let Ok(u) = ud.borrow::<LuaUDim2>() {
                    return Some(Self::UDim2(u.clone()));
                }
                if let Ok(v) = ud.borrow::<LuaVector2>() {
                    return Some(Self::Vector2(v.clone()));
                }
                if let Ok(v) = ud.borrow::<LuaVector3>() {
                    return Some(Self::Vector3(v.clone()));
                }
                None
            }
            _ => None,
        }
    }

    /// Convert the Rust math value into Lua value to update the instance
    pub fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        match self {
            Self::Float(v) => Ok(mlua::Value::Number(v as f64)),
            Self::Color3(v) => Ok(mlua::Value::UserData(lua.create_userdata(v)?)),
            Self::UDim2(v) => Ok(mlua::Value::UserData(lua.create_userdata(v)?)),
            Self::Vector2(v) => Ok(mlua::Value::UserData(lua.create_userdata(v)?)),
            Self::Vector3(v) => Ok(mlua::Value::UserData(lua.create_userdata(v)?)),
        }
    }

    /// Pure interpolation (lerp) for better performance
    pub fn lerp(&self, target: &Self, alpha: f32) -> Self {
        match (self, target) {
            (Self::Float(a), Self::Float(b)) => Self::Float(a + (b - a) * alpha),
            (Self::Color3(a), Self::Color3(b)) => Self::Color3(LuaColor3 {
                r: a.r + (b.r - a.r) * alpha,
                g: a.g + (b.g - a.g) * alpha,
                b: a.b + (b.b - a.b) * alpha,
            }),
            (Self::UDim2(a), Self::UDim2(b)) => Self::UDim2(LuaUDim2 {
                x_scale: a.x_scale + (b.x_scale - a.x_scale) * alpha,
                x_offset: a.x_offset + (b.x_offset - a.x_offset) * alpha,
                y_scale: a.y_scale + (b.y_scale - a.y_scale) * alpha,
                y_offset: a.y_offset + (b.y_offset - a.y_offset) * alpha,
            }),
            (Self::Vector2(a), Self::Vector2(b)) => Self::Vector2(LuaVector2 {
                x: a.x + (b.x - a.x) * alpha,
                y: a.y + (b.y - a.y) * alpha,
            }),
            (Self::Vector3(a), Self::Vector3(b)) => Self::Vector3(LuaVector3 {
                x: a.x + (b.x - a.x) * alpha,
                y: a.y + (b.y - a.y) * alpha,
                z: a.z + (b.z - a.z) * alpha,
            }),
            _ => self.clone(),
        }
    }

    /// Return the name of the type expected
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Float(_) => "number",
            Self::Color3(_) => "Color3",
            Self::UDim2(_) => "UDim2",
            Self::Vector2(_) => "Vector2",
            Self::Vector3(_) => "Vector3",
        }
    }
}

/// Compute the animation courb (roblox standard)
pub fn calculate_alpha(
    time_elapsed: f32,
    duration: f32,
    style: EasingStyle,
    dir: EasingDirection,
) -> f32 {
    if duration == 0.0 {
        return 1.0;
    }
    let t = (time_elapsed / duration).clamp(0.0, 1.0);

    match style {
        EasingStyle::Linear => t,
        EasingStyle::Sine => {
            use std::f32::consts::PI;
            match dir {
                EasingDirection::In => 1.0 - (t * PI / 2.0).cos(),
                EasingDirection::Out => (t * PI / 2.0).sin(),
                EasingDirection::InOut => -0.5 * ((t * PI).cos() - 1.0),
            }
        }
        EasingStyle::Quad => match dir {
            EasingDirection::In => t * t,
            EasingDirection::Out => t * (2.0 - t),
            EasingDirection::InOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        },
        _ => t,
    }
}
