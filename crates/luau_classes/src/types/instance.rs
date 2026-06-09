use bevy::prelude::*;
use luau_runtime::bridge::{handle::HandleMap, queue::EngineQueue};
use mlua::ObjectLike;

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

    pub fn set_parent(&mut self, parent: Option<mlua::AnyUserData>) {
        let new_parent_handle = parent.and_then(|ud| instance_handle_from_any(&ud));
        self.parent_handle = new_parent_handle;
        let handle = self.handle;

        self.queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut bevy::prelude::World| {
                let map = w.resource::<HandleMap>();
                let child_e = map.get_entity(handle);
                let parent_e = new_parent_handle.and_then(|h| map.get_entity(h));

                if let Some(child) = child_e {
                    if let Ok(mut e_mut) = w.get_entity_mut(child) {
                        e_mut.remove_parent_in_place();

                        if parent_e.is_some() {
                            e_mut.insert(Visibility::Inherited);
                        } else {
                            e_mut.insert(Visibility::Hidden);
                        }
                    }

                    if let Some(parent) = parent_e {
                        if let Ok(mut p_mut) = w.get_entity_mut(parent) {
                            p_mut.add_child(child);
                        }
                    }
                }
            }));
    }

    pub fn prepare_clone(&self) -> Self {
        let mut c = self.clone();
        c.handle = luau_runtime::bridge::handle::next_handle();
        c.parent_handle = None;
        c.children_handles.clear();
        c
    }

    pub fn spawn_base_entity(&self, w: &mut World) -> Entity {
        let entity = w
            .spawn((
                Transform::default(),
                Visibility::Hidden,
                luau_runtime::bridge::handle::LuauHandle(self.handle),
            ))
            .id();
        w.resource_mut::<luau_runtime::bridge::handle::HandleMap>()
            .insert(self.handle, entity, None);
        entity
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
macro_rules! impl_lua_clone {
    ($methods:ident) => {
        $methods.add_method("__clone_data", |lua, this, ()| {
            use crate::types::instance::CloneableInstance;
            let mut new_instance = this.clone();

            *new_instance.base_mut() = new_instance.base().prepare_clone();

            new_instance.on_cloned(lua)?;

            let c = new_instance.clone();
            new_instance.base().queue.0.lock().unwrap().push(Box::new(
                move |w: &mut bevy::prelude::World| {
                    let entity = c.base().spawn_base_entity(w);
                    c.apply_bevy_components(entity, w);
                },
            ));

            Ok(lua.create_userdata(new_instance)?)
        });

        $methods.add_method("__get_children", |_, this, ()| {
            use crate::types::instance::CloneableInstance;
            Ok(this.base().children_handles.clone())
        });

        $methods.add_method("Clone", |lua, this, ()| {
            use crate::types::instance::CloneableInstance;
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            let original_ud: mlua::AnyUserData = cache.get(this.base().handle)?;
            crate::types::instance::universal_clone(lua, &original_ud, None)
        });
    };
}

pub fn instance_handle_from_any(ud: &mlua::AnyUserData) -> Option<u64> {
    if let Ok(p) = ud.borrow::<crate::instances::part::LuaPart>() {
        return Some(p.0.base.handle);
    }
    if let Ok(f) = ud.borrow::<crate::instances::frame::LuaFrame>() {
        return Some(f.base.handle);
    }
    if let Ok(sg) = ud.borrow::<crate::instances::screen_gui::LuaScreenGui>() {
        return Some(sg.base.handle);
    }
    if let Ok(rb) = ud.borrow::<crate::instances::rigidbody::LuaRigidbody>() {
        return Some(rb.base.handle);
    }
    if let Ok(cd) = ud.borrow::<crate::instances::collider::LuaCollider>() {
        return Some(cd.base.handle);
    }
    if let Ok(ws) = ud.borrow::<crate::instances::workspace::LuaWorkspace>() {
        return Some(ws.base.handle);
    }
    None
}

pub fn universal_clone(
    lua: &mlua::Lua,
    original: &mlua::AnyUserData,
    parent: Option<mlua::AnyUserData>,
) -> mlua::Result<mlua::AnyUserData> {
    let cloned: mlua::AnyUserData = original.call_method("__clone_data", ())?;
    let handle = crate::types::instance::instance_handle_from_any(&cloned).unwrap();
    lua.named_registry_value::<mlua::Table>("__instance_cache")?
        .set(handle, cloned.clone())?;

    if let Some(p) = parent {
        cloned.set("Parent", p)?;
    }

    let children: Vec<u64> = original.call_method("__get_children", ())?;
    for child_handle in children {
        if let Ok(cache) = lua.named_registry_value::<mlua::Table>("__instance_cache") {
            if let Ok(child_ud) = cache.get::<mlua::AnyUserData>(child_handle) {
                let _ = universal_clone(lua, &child_ud, Some(cloned.clone()))?;
            }
        }
    }
    Ok(cloned)
}
