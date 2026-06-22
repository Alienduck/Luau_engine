use bevy::prelude::Component;

/// Struct to handle easier the bloom script side
/// [TODO]: found a way to handle without the custom struct
#[derive(Component)]
pub struct LuauBloom {
    pub intensity: f32,
    pub size: f32,
    pub threshold: f32,
}
