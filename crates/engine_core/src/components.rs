use bevy::{color::Color, math::Vec3, prelude::Component};

/// Struct to handle easier the bloom script side
/// [TODO]: found a way to handle without the custom struct
#[derive(Component)]
pub struct LuauBloom {
    pub intensity: f32,
    pub size: f32,
    pub threshold: f32,
}

#[derive(Component, Clone)]
pub struct LuauAtmosphere {
    pub density: f32,
    pub color: Color,
    pub decay: Color,
    pub glare: f32,
    pub haze: f32,
}

#[derive(Component, Default)]
pub struct LuauCharacterController {
    pub velocity: Vec3,
}
