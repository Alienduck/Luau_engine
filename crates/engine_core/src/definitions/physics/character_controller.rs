use bevy::prelude::*;
use bevy_tnua::prelude::*;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum CharacterControllerScheme {
    Jumping(TnuaBuiltinJump),
}

#[derive(Component)]
pub struct LuauCharacterController {
    pub walk_speed: f32,
    pub jump_power: f32,
    pub move_direction: Vec3,
    pub jump: bool,
    pub hip_height: f32,
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

impl Default for LuauCharacterController {
    fn default() -> Self {
        Self {
            walk_speed: 16.0,
            jump_power: 24.0,
            move_direction: Vec3::default(),
            jump: false,
            hip_height: 0.1,
            custom_forward_button: None,
            custom_behind_button: None,
            custom_left_button: None,
            custom_right_button: None,
            custom_jump_button: None,
        }
    }
}
