use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource)]
pub struct PhysicsCollisionGroups {
    pub groups: HashMap<String, u32>,
    pub masks: [u32; 32],
    pub next_id: u32,
}

impl Default for PhysicsCollisionGroups {
    fn default() -> Self {
        let mut groups = HashMap::new();
        groups.insert("Default".to_string(), 0);
        Self {
            groups,
            masks: [u32::MAX; 32],
            next_id: 1,
        }
    }
}
