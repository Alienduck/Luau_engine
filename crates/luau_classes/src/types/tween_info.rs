use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::{Lua, UserData, UserDataFields};

use crate::types::enums::{EasingDirection, EasingStyle};

#[derive(Clone, Debug)]
pub struct LuaTweenInfo {
    pub time: f32,
    pub easing_style: EasingStyle,
    pub easing_direction: EasingDirection,
    pub repeat_count: i32,
    pub reverses: bool,
    pub delay_time: f32,
}

impl Default for LuaTweenInfo {
    fn default() -> Self {
        Self {
            time: 1.0,
            easing_style: EasingStyle::Quad,
            easing_direction: EasingDirection::Out,
            repeat_count: 0,
            reverses: false,
            delay_time: 0.0,
        }
    }
}

impl UserData for LuaTweenInfo {
    fn add_fields<M: UserDataFields<Self>>(fields: &mut M) {
        fields.add_field_method_get("Time", |_, this| Ok(this.time));
        fields.add_field_method_get("EasingStyle", |_, this| Ok(this.easing_style as u8));
        fields.add_field_method_get("EasingDirection", |_, this| Ok(this.easing_direction as u8));
        fields.add_field_method_get("RepeatCount", |_, this| Ok(this.repeat_count));
        fields.add_field_method_get("Reverses", |_, this| Ok(this.reverses));
        fields.add_field_method_get("DelayTime", |_, this| Ok(this.delay_time));
    }
}

pub struct TweenInfoModule;

impl LuaModule for TweenInfoModule {
    fn name() -> &'static str {
        "TweenInfo"
    }
    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let tween_info = lua.create_table()?;
        tween_info.set(
            "new",
            lua.create_function(
                |_,
                 (time, style, dir, repeat, rev, delay): (
                    Option<f32>,
                    Option<u8>,
                    Option<u8>,
                    Option<i32>,
                    Option<bool>,
                    Option<f32>,
                )| {
                    Ok(LuaTweenInfo {
                        time: time.unwrap_or(1.0),
                        easing_style: match style.unwrap_or(EasingStyle::Quad as u8) {
                            0 => EasingStyle::Linear,
                            1 => EasingStyle::Sine,
                            2 => EasingStyle::Quad,
                            3 => EasingStyle::Cubic,
                            4 => EasingStyle::Quart,
                            5 => EasingStyle::Quint,
                            6 => EasingStyle::Bounce,
                            7 => EasingStyle::Elastic,
                            8 => EasingStyle::Exponential,
                            9 => EasingStyle::Circular,
                            10 => EasingStyle::Back,
                            _ => EasingStyle::Quad,
                        },
                        easing_direction: match dir.unwrap_or(EasingDirection::Out as u8) {
                            0 => EasingDirection::In,
                            1 => EasingDirection::Out,
                            2 => EasingDirection::InOut,
                            _ => EasingDirection::Out,
                        },
                        repeat_count: repeat.unwrap_or(0),
                        reverses: rev.unwrap_or(false),
                        delay_time: delay.unwrap_or(0.0),
                    })
                },
            )?,
        )?;

        lua.globals().set("TweenInfo", tween_info)?;

        Ok(())
    }
}
