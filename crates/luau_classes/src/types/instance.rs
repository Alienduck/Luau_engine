use bevy::prelude::*;
use luau_runtime::bridge::{handle::HandleMap, queue::EngineQueue};
use mlua::ObjectLike;

/// Shared base data embedded in every Luau-facing instance type.
///
/// Mirrors the Roblox `Instance` base class: every object has a handle (stable
/// u64 that maps to a Bevy entity), a name, a class tag, parent/children
/// tracking, and access to the engine queue.
#[derive(Clone)]
pub struct InstanceData {
    /// Stable identifier used to look up the Bevy entity in [`HandleMap`].
    pub handle: u64,
    /// Human-readable name (defaults to `class_name`).
    pub name: String,
    /// Static class tag (e.g. `"Part"`, `"Collider"`).
    pub class_name: &'static str,
    /// Handle of the parent instance, if any.
    pub parent_handle: Option<u64>,
    /// Handles of all direct children.
    pub children_handles: Vec<u64>,
    /// Write-only bridge to the Bevy world.
    pub queue: EngineQueue,
}

impl InstanceData {
    pub fn new(handle: u64, queue: EngineQueue, class_name: &'static str) -> Self {
        Self {
            handle,
            name: class_name.to_string(),
            class_name,
            parent_handle: None,
            children_handles: Vec::new(),
            queue,
        }
    }

    pub fn set_name(&mut self, v: String) {
        self.name = v;
    }

    /// Updates the Luau-side parent/child caches and enqueues the matching
    /// Bevy hierarchy mutation.
    ///
    /// No-op when the new parent is identical to the current one.
    pub fn set_parent(&mut self, lua: &mlua::Lua, parent: Option<mlua::AnyUserData>) {
        let new_parent_handle = parent.as_ref().and_then(|ud| instance_handle_from_any(ud));
        let old_parent_handle = self.parent_handle;
        let self_handle = self.handle;

        if old_parent_handle == new_parent_handle {
            return;
        }

        // Update Luau-side children lists via the instance cache.
        if let Ok(cache) = lua.named_registry_value::<mlua::Table>("__instance_cache") {
            if let Some(old_h) = old_parent_handle {
                if let Ok(old_ud) = cache.get::<mlua::AnyUserData>(old_h) {
                    let _ = old_ud.call_method::<()>("__remove_child_handle", self_handle);
                }
            }
            if let Some(new_ud) = &parent {
                let _ = new_ud.call_method::<()>("__add_child_handle", self_handle);
            }
        }

        self.parent_handle = new_parent_handle;

        // Enqueue the Bevy hierarchy mutation.
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                let map = w.resource::<HandleMap>();
                let child_e = map.get_entity(self_handle);
                let parent_e = new_parent_handle.and_then(|h| map.get_entity(h));

                if let Some(child) = child_e {
                    if let Ok(mut e_mut) = w.get_entity_mut(child) {
                        e_mut.remove_parent_in_place();
                        e_mut.insert(if parent_e.is_some() {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        });
                    }
                    if let Some(parent) = parent_e {
                        if let Ok(mut p_mut) = w.get_entity_mut(parent) {
                            p_mut.add_child(child);
                        }
                    }
                }
            }));
    }

    /// Prepares a shallow clone of this data with a fresh handle and cleared
    /// parent/children state, ready to be registered as a new instance.
    pub fn prepare_clone(&self) -> Self {
        let mut c = self.clone();
        c.handle = luau_runtime::bridge::handle::next_handle();
        c.parent_handle = None;
        c.children_handles.clear();
        c
    }

    /// Spawns the minimal Bevy entity: `Transform`, `Visibility::Hidden`, and
    /// a [`LuauHandle`] tag. Registers the entity in [`HandleMap`].
    ///
    /// Called from [`CloneableInstance::apply_bevy_components`] implementations
    /// (which add type-specific components on top).
    pub fn spawn_base_entity(&self, w: &mut World) -> Entity {
        let entity = w
            .spawn((
                Transform::default(),
                Visibility::Hidden,
                luau_runtime::bridge::handle::LuauHandle(self.handle),
            ))
            .id();
        w.resource_mut::<HandleMap>()
            .insert(self.handle, entity, None);
        entity
    }

    /// Enqueues recursive destruction of this instance's Bevy entity and
    /// removal of its entry from [`HandleMap`].
    ///
    /// Uses [`EntityWorldMut::despawn`] which is recursive in Bevy 0.18+,
    /// so child entities are automatically cleaned up.
    pub fn destroy(&self) {
        let h = self.handle;
        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(entry) = w.resource_mut::<HandleMap>().remove(h) {
                    // despawn via EntityWorldMut is recursive — children are removed too.
                    if let Ok(e_mut) = w.get_entity_mut(entry.entity) {
                        e_mut.despawn();
                    }
                }
            }));
    }
}

// CloneableInstance trait

/// Implemented by every concrete Luau instance type.
///
/// Provides access to [`InstanceData`] (required) and hooks for post-clone
/// state fixup and Bevy component insertion.
pub trait CloneableInstance: Clone + mlua::UserData {
    fn base(&self) -> &InstanceData;
    fn base_mut(&mut self) -> &mut InstanceData;

    /// Called after a clone is created, before it is registered or spawned.
    ///
    /// Use this to regenerate any ids that must be unique per-instance (e.g.
    /// signal ids).  Default impl is a no-op.
    fn on_cloned(&mut self, _lua: &mlua::Lua) -> mlua::Result<()> {
        Ok(())
    }

    /// Inserts type-specific Bevy components onto `entity` (meshes, materials,
    /// physics bodies, etc.).  The base transform/visibility/handle are already
    /// present from [`InstanceData::spawn_base_entity`].
    fn apply_bevy_components(&self, entity: Entity, w: &mut World);
}

/// Injects the full set of shared Luau instance methods into a [`UserDataMethods`]
/// implementation.
///
/// Covers:
/// - `Clone` — deep-clones the instance tree
/// - `Destroy` — recursively despawns
/// - `GetChildren` — returns array of direct children
/// - `GetDescendants` — returns array of all descendants
/// - `FindFirstChild(name)` — returns first child with matching name
/// - `IsDescendantOf(ancestor)` — walks up the parent chain
/// - Internal helpers: `__clone_data`, `__get_children`,
///   `__add_child_handle`, `__remove_child_handle`
///
/// Every concrete instance type must call this macro from its
/// `UserData::add_methods` implementation instead of scattering identical
/// boilerplate across files.
#[macro_export]
macro_rules! impl_instance_userdata {
    ($methods:ident) => {
        $methods.add_method("__clone_data", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let mut cloned = this.clone();
            *cloned.base_mut() = cloned.base().prepare_clone();
            cloned.on_cloned(lua)?;
            let c = cloned.clone();
            cloned.base().queue.0.lock().unwrap().push(Box::new(
                move |w: &mut bevy::prelude::World| {
                    let entity = c.base().spawn_base_entity(w);
                    c.apply_bevy_components(entity, w);
                },
            ));
            Ok(lua.create_userdata(cloned)?)
        });

        $methods.add_method("__get_children", |_, this, ()| {
            use $crate::types::instance::CloneableInstance;
            Ok(this.base().children_handles.clone())
        });

        $methods.add_method_mut("__add_child_handle", |_, this, child_handle: u64| {
            use $crate::types::instance::CloneableInstance;
            this.base_mut().children_handles.push(child_handle);
            Ok(())
        });

        $methods.add_method_mut("__remove_child_handle", |_, this, child_handle: u64| {
            use $crate::types::instance::CloneableInstance;
            this.base_mut()
                .children_handles
                .retain(|&h| h != child_handle);
            Ok(())
        });

        $methods.add_method("Clone", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            let original_ud: mlua::AnyUserData = cache.get(this.base().handle)?;
            $crate::types::instance::universal_clone(lua, &original_ud, None)
        });

        $methods.add_method("Destroy", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            $crate::types::instance::recursive_destroy(lua, &cache, this.base())?;
            Ok(())
        });

        $methods.add_method("GetChildren", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            let result = lua.create_table()?;
            for (i, &handle) in this.base().children_handles.iter().enumerate() {
                if let Ok(ud) = cache.get::<mlua::AnyUserData>(handle) {
                    result.set(i + 1, ud)?;
                }
            }
            Ok(result)
        });

        $methods.add_method("GetDescendants", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            let result = lua.create_table()?;
            let mut idx = 1usize;
            $crate::types::instance::collect_descendants(
                &cache,
                &this.base().children_handles,
                &result,
                &mut idx,
            )?;
            Ok(result)
        });

        $methods.add_method("FindFirstChild", |lua, this, name: String| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            for &handle in &this.base().children_handles {
                if let Ok(ud) = cache.get::<mlua::AnyUserData>(handle) {
                    if let Some(n) = $crate::types::instance::instance_name_from_any(&ud) {
                        if n == name {
                            return Ok(Some(ud));
                        }
                    }
                }
            }
            Ok(None)
        });

        $methods.add_method(
            "IsDescendantOf",
            |lua, this, ancestor: mlua::AnyUserData| {
                use $crate::types::instance::CloneableInstance;
                let target_handle =
                    match $crate::types::instance::instance_handle_from_any(&ancestor) {
                        Some(h) => h,
                        None => return Ok(false),
                    };
                let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
                let mut current = this.base().parent_handle;
                while let Some(h) = current {
                    if h == target_handle {
                        return Ok(true);
                    }
                    current = cache.get::<mlua::AnyUserData>(h).ok().and_then(|ud| {
                        $crate::types::instance::instance_parent_handle_from_any(&ud)
                    });
                }
                Ok(false)
            },
        );
        $methods.add_method("__get_handle", |_, this, ()| {
            use $crate::types::instance::CloneableInstance;
            Ok(this.base().handle)
        });
        $methods.add_method("__get_name", |_, this, ()| {
            use $crate::types::instance::CloneableInstance;
            Ok(this.base().name.clone())
        });
        $methods.add_method("__get_parent_handle", |_, this, ()| {
            use $crate::types::instance::CloneableInstance;
            Ok(this.base().parent_handle)
        });
        $methods.add_method("__destroy_internal", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            $crate::types::instance::recursive_destroy(lua, &cache, this.base())?;
            Ok(())
        });
    };
}

/// Returns the handle of `ud` by trying each known concrete instance type.
///
/// O(N) in the number of types — acceptable given the small, fixed set.
pub fn instance_handle_from_any(ud: &mlua::AnyUserData) -> Option<u64> {
    ud.call_method::<u64>("__get_handle", ()).ok()
}

/// Returns the `name` field of `ud`, or `None` if the type is unrecognised.
pub fn instance_name_from_any(ud: &mlua::AnyUserData) -> Option<String> {
    ud.call_method::<String>("__get_name", ()).ok()
}

/// Returns the `parent_handle` of `ud`, or `None` if the type is unrecognised.
pub fn instance_parent_handle_from_any(ud: &mlua::AnyUserData) -> Option<u64> {
    ud.call_method::<Option<u64>>("__get_parent_handle", ())
        .ok()
        .flatten()
}

/// Recursively collects all descendants into `result`, depth-first.
pub fn collect_descendants(
    cache: &mlua::Table,
    children: &[u64],
    result: &mlua::Table,
    idx: &mut usize,
) -> mlua::Result<()> {
    for &handle in children {
        if let Ok(ud) = cache.get::<mlua::AnyUserData>(handle) {
            result.set(*idx, ud.clone())?;
            *idx += 1;
            let grandchildren: Vec<u64> = ud.call_method("__get_children", ())?;
            collect_descendants(cache, &grandchildren, result, idx)?;
        }
    }
    Ok(())
}

/// Recursively destroys an instance subtree, removing each node from
/// `cache` and enqueuing a Bevy despawn.
///
/// Children are destroyed bottom-up so parent entities are still alive when
/// children are unlinked, but since [`InstanceData::destroy`] uses a
/// recursive Bevy despawn the order doesn't technically matter for the ECS.
pub fn recursive_destroy(
    _lua: &mlua::Lua,
    cache: &mlua::Table,
    data: &InstanceData,
) -> mlua::Result<()> {
    for &child_handle in &data.children_handles {
        if let Ok(child_ud) = cache.get::<mlua::AnyUserData>(child_handle) {
            let _ = child_ud.call_method::<()>("__destroy_internal", ());
        }
        let _ = cache.set(child_handle, mlua::Value::Nil);
    }
    if let Some(parent_h) = data.parent_handle {
        if let Ok(parent_ud) = cache.get::<mlua::AnyUserData>(parent_h) {
            let _ = parent_ud.call_method::<()>("__remove_child_handle", data.handle);
        }
    }
    data.destroy();
    let _ = cache.set(data.handle, mlua::Value::Nil);
    Ok(())
}

/// Deep-clones `original` and recursively clones all its descendants.
///
/// If `parent` is provided the clone is immediately parented to it.
/// Every cloned node is inserted into `__instance_cache`.
pub fn universal_clone(
    lua: &mlua::Lua,
    original: &mlua::AnyUserData,
    parent: Option<mlua::AnyUserData>,
) -> mlua::Result<mlua::AnyUserData> {
    let cloned: mlua::AnyUserData = original.call_method("__clone_data", ())?;
    let handle =
        instance_handle_from_any(&cloned).expect("__clone_data must return a known instance type");

    lua.named_registry_value::<mlua::Table>("__instance_cache")?
        .set(handle, cloned.clone())?;

    if let Some(p) = parent {
        cloned.set("Parent", p)?;
    }

    let children: Vec<u64> = original.call_method("__get_children", ())?;
    let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;

    for child_handle in children {
        if let Ok(child_ud) = cache.get::<mlua::AnyUserData>(child_handle) {
            universal_clone(lua, &child_ud, Some(cloned.clone()))?;
        }
    }
    Ok(cloned)
}
