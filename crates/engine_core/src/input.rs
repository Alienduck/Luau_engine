use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// A physical key (keyboard or mouse button) that can be bound to a named action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoundKey {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

/// Runtime state of every registered action binding.
///
/// Populated each frame by [`update_action_states`]; consumed by camera and
/// any other system that needs logical-action queries instead of raw key checks.
#[derive(Resource)]
pub struct ActionMap {
    /// Mapping from action name → list of physical keys that trigger it.
    pub bindings: HashMap<String, Vec<BoundKey>>,
    /// Actions whose bound key is currently held.
    pub active_actions: HashSet<String>,
    /// Actions whose bound key transitioned to pressed this frame.
    pub just_pressed_actions: HashSet<String>,
    /// Actions whose bound key transitioned to released this frame.
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
    /// Returns `true` if any bound key for `action` is currently held.
    pub fn is_pressed(&self, action: &str) -> bool {
        self.active_actions.contains(action)
    }

    /// Returns `true` if any bound key for `action` was pressed this frame.
    pub fn just_pressed(&self, action: &str) -> bool {
        self.just_pressed_actions.contains(action)
    }

    /// Returns `true` if any bound key for `action` was released this frame.
    pub fn just_released(&self, action: &str) -> bool {
        self.just_released_actions.contains(action)
    }
}

/// Bevy system — refreshes [`ActionMap`] state from raw [`ButtonInput`] resources.
///
/// Must run every frame, before any system that queries the action map.
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
