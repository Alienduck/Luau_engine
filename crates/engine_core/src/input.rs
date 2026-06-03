use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoundKey {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

#[derive(Resource)]
pub struct ActionMap {
    pub bindings: HashMap<String, Vec<BoundKey>>,
    pub active_actions: HashSet<String>,
    pub just_pressed_actions: HashSet<String>,
    pub just_released_actions: HashSet<String>,
}

impl Default for ActionMap {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(
            "CameraLook".into(),
            vec![BoundKey::Mouse(MouseButton::Right)],
        );
        bindings.insert(
            "CameraToggleLock".into(),
            vec![BoundKey::Keyboard(KeyCode::ShiftLeft)],
        );

        Self {
            bindings,
            active_actions: HashSet::new(),
            just_pressed_actions: HashSet::new(),
            just_released_actions: HashSet::new(),
        }
    }
}

impl ActionMap {
    pub fn is_pressed(&self, action: &str) -> bool {
        self.active_actions.contains(action)
    }
    pub fn just_pressed(&self, action: &str) -> bool {
        self.just_pressed_actions.contains(action)
    }
    pub fn just_released(&self, action: &str) -> bool {
        self.just_released_actions.contains(action)
    }
}

pub fn update_action_states(
    mut action_map: ResMut<ActionMap>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    action_map.active_actions.clear();
    action_map.just_pressed_actions.clear();
    action_map.just_released_actions.clear();

    for (action, bound_keys) in action_map.bindings.clone().iter() {
        for key in bound_keys {
            let (pressed, just_pressed, just_released) = match key {
                BoundKey::Keyboard(k) => (
                    keys.pressed(*k),
                    keys.just_pressed(*k),
                    keys.just_released(*k),
                ),
                BoundKey::Mouse(m) => (
                    mouse.pressed(*m),
                    mouse.just_pressed(*m),
                    mouse.just_released(*m),
                ),
            };
            if pressed {
                action_map.active_actions.insert(action.clone());
            }
            if just_pressed {
                action_map.just_pressed_actions.insert(action.clone());
            }
            if just_released {
                action_map.just_released_actions.insert(action.clone());
            }
        }
    }
}
