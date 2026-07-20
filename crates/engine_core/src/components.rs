use bevy::prelude::*;

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
    pub walk_speed: f32,
    pub jump_power: f32,
    pub move_direction: Vec3,
    pub jump: bool,
    pub custom_forward_button: Option<KeyCode>,
    pub custom_behind_button: Option<KeyCode>,
    pub custom_left_button: Option<KeyCode>,
    pub custom_right_button: Option<KeyCode>,
    pub custom_jump_button: Option<KeyCode>,
}

impl LuauCharacterController {
    pub fn custom_inputs_or_default(&self) -> (KeyCode, KeyCode, KeyCode, KeyCode, KeyCode) {
        (
            self.custom_forward_button.unwrap_or(KeyCode::KeyW),
            self.custom_behind_button.unwrap_or(KeyCode::KeyS),
            self.custom_left_button.unwrap_or(KeyCode::KeyA),
            self.custom_right_button.unwrap_or(KeyCode::KeyD),
            self.custom_jump_button.unwrap_or(KeyCode::Space),
        )
    }
}
