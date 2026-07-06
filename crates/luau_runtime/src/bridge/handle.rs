use bevy::prelude::*;
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
pub struct HandleMap {
    entries: Vec<Option<EntityEntry>>,
}

impl HandleMap {
    pub fn insert(
        &mut self,
        handle: u64,
        entity: Entity,
        material: Option<Handle<StandardMaterial>>,
    ) {
        let index = handle as usize;
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, || None);
        }
        self.entries[index] = Some(EntityEntry { entity, material });
    }

    #[inline(always)]
    pub fn get_entity(&self, handle: u64) -> Option<Entity> {
        let index = handle as usize;
        self.entries
            .get(index)
            .and_then(|entry| entry.as_ref().map(|e| e.entity))
    }

    #[inline(always)]
    pub fn get_material(&self, handle: u64) -> Option<Handle<StandardMaterial>> {
        let index = handle as usize;
        self.entries
            .get(index)
            .and_then(|entry| entry.as_ref().and_then(|e| e.material.clone()))
    }

    pub fn remove(&mut self, handle: u64) -> Option<EntityEntry> {
        let index = handle as usize;
        if index < self.entries.len() {
            self.entries[index].take()
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct LuauHandle(pub u64);
