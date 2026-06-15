use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

/// Allocate a new unique handle. Thread-safe, never returns 0.
#[inline]
pub fn next_handle() -> u64 {
    NEXT_HANDLE.fetch_add(1, Ordering::Relaxed)
}

pub struct EntityEntry {
    pub entity: Entity,
    /// None for entities without a StandardMaterial.
    pub material: Option<Handle<StandardMaterial>>,
}

/// Maps Luau handles (u64) to Bevy entities.
#[derive(Resource, Default)]
pub struct HandleMap(pub HashMap<u64, EntityEntry>);

impl HandleMap {
    pub fn insert(
        &mut self,
        handle: u64,
        entity: Entity,
        material: Option<Handle<StandardMaterial>>,
    ) {
        self.0.insert(handle, EntityEntry { entity, material });
    }

    pub fn get_entity(&self, handle: u64) -> Option<Entity> {
        self.0.get(&handle).map(|e| e.entity)
    }

    pub fn get_material(&self, handle: u64) -> Option<Handle<StandardMaterial>> {
        self.0.get(&handle).and_then(|e| e.material.clone())
    }

    pub fn remove(&mut self, handle: u64) -> Option<EntityEntry> {
        self.0.remove(&handle)
    }
}

#[derive(Component)]
pub struct LuauHandle(pub u64);
