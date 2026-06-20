use bevy::prelude::*;
use luau_runtime::bridge::{
    handle::HandleMap,
    queue::{EngineCommand, EngineQueue},
};
use mlua::ObjectLike;

/// Shared base data embedded in every Luau-facing instance type.
#[derive(Clone)]
pub struct InstanceData {
    pub handle: u64,
    pub name: String,
    pub class_name: &'static str,
    pub parent_handle: Option<u64>,
    pub children_handles: Vec<u64>,
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

    /// Updates the Luau-side parent/child caches and enqueues a typed
    /// [`EngineCommand::SetParent`] — no closure, no Box.
    pub fn set_parent(&mut self, lua: &mlua::Lua, parent: Option<mlua::AnyUserData>) {
        let new_parent_handle = parent.as_ref().and_then(|ud| instance_handle_from_any(ud));
        let old_parent_handle = self.parent_handle;
        let self_handle = self.handle;

        if old_parent_handle == new_parent_handle {
            return;
        }

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

        self.queue.push(EngineCommand::SetParent {
            child_handle: self_handle,
            parent_handle: new_parent_handle,
        });
    }

    pub fn prepare_clone(&self) -> Self {
        let mut c = self.clone();
        c.handle = luau_runtime::bridge::handle::next_handle();
        c.parent_handle = None;
        c.children_handles.clear();
        c
    }

    /// Spawns the minimal Bevy entity and registers it in [`HandleMap`].
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

    /// Enqueues a typed [`EngineCommand::Despawn`] — no closure, no Box.
    pub fn destroy(&self) {
        self.queue.push(EngineCommand::Despawn {
            handle: self.handle,
        });
    }
}

pub trait CloneableInstance: Clone + mlua::UserData {
    fn base(&self) -> &InstanceData;
    fn base_mut(&mut self) -> &mut InstanceData;

    fn on_cloned(&mut self, _lua: &mlua::Lua) -> mlua::Result<()> {
        Ok(())
    }

    fn apply_bevy_components(&self, entity: Entity, w: &mut World);
}

#[macro_export]
macro_rules! impl_instance_userdata {
    ($methods:ident) => {
        $methods.add_method("__clone_data", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let mut cloned = this.clone();
            *cloned.base_mut() = cloned.base().prepare_clone();
            cloned.on_cloned(lua)?;
            let c = cloned.clone();
            let entity_handle = c.base().handle;
            let queue = c.base().queue.clone();
            queue.push_raw(move |w: &mut bevy::prelude::World| {
                let entity = c.base().spawn_base_entity(w);
                c.apply_bevy_components(entity, w);
                let _ = entity_handle;
            });
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
            $crate::types::instance::recursive_destroy(lua, &cache, this.base(), true)?;
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

        $methods.add_method("__destroy_internal_no_notify", |lua, this, ()| {
            use $crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            $crate::types::instance::recursive_destroy(lua, &cache, this.base(), false)?;
            Ok(())
        });
    };
}

#[macro_export]
macro_rules! impl_base_instance_fields {
    ($fields:ident) => {
        $fields.add_field_method_get("Name", |_, this| {
            use $crate::types::instance::CloneableInstance;
            Ok(this.base().name.clone())
        });
        $fields.add_field_method_get("ClassName", |_, this| {
            use $crate::types::instance::CloneableInstance;
            Ok(this.base().class_name)
        });
        $fields.add_field_method_set("Name", |_, this, v: String| {
            use $crate::types::instance::CloneableInstance;
            this.base_mut().set_name(v);
            Ok(())
        });
        $fields.add_field_method_get("Parent", |lua, this| {
            use $crate::types::instance::CloneableInstance;
            if let Some(parent_handle) = this.base().parent_handle {
                let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
                Ok(cache.get::<Option<mlua::AnyUserData>>(parent_handle)?)
            } else {
                Ok(None)
            }
        });
        $fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            use $crate::types::instance::CloneableInstance;
            this.base_mut().set_parent(lua, parent);
            Ok(())
        });
    };
}

pub fn instance_handle_from_any(ud: &mlua::AnyUserData) -> Option<u64> {
    ud.call_method::<u64>("__get_handle", ()).ok()
}

pub fn instance_name_from_any(ud: &mlua::AnyUserData) -> Option<String> {
    ud.call_method::<String>("__get_name", ()).ok()
}

pub fn instance_parent_handle_from_any(ud: &mlua::AnyUserData) -> Option<u64> {
    ud.call_method::<Option<u64>>("__get_parent_handle", ())
        .ok()
        .flatten()
}

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

pub fn recursive_destroy(
    _lua: &mlua::Lua,
    cache: &mlua::Table,
    data: &InstanceData,
    notify_parent: bool,
) -> mlua::Result<()> {
    let children = data.children_handles.clone();

    for child_handle in children {
        if let Ok(child_ud) = cache.get::<mlua::AnyUserData>(child_handle) {
            let _ = child_ud.call_method::<()>("__destroy_internal_no_notify", ());
        }
        let _ = cache.set(child_handle, mlua::Value::Nil);
    }

    if notify_parent {
        if let Some(parent_h) = data.parent_handle {
            if let Ok(parent_ud) = cache.get::<mlua::AnyUserData>(parent_h) {
                let _ = parent_ud.call_method::<()>("__remove_child_handle", data.handle);
            }
        }
    }

    data.destroy();
    let _ = cache.set(data.handle, mlua::Value::Nil);

    Ok(())
}

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
