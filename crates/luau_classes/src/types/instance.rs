use bevy::prelude::*;
use luau_runtime::bridge::{handle::HandleMap, queue::EngineQueue};

use crate::{instances::part::LuaPart, types::signal::LuaSignal};

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
    original_ud: &mlua::AnyUserData,
    parent_for_clone: Option<mlua::AnyUserData>,
) -> mlua::Result<mlua::AnyUserData> {
    let cloned_ud = if let Ok(part) = original_ud.borrow::<LuaPart>() {
        let new_handle = luau_runtime::bridge::handle::next_handle();
        let new_sig = LuaSignal::new(lua)?.id;

        let new_data = part.0.clone_with_new_ids(new_handle, new_sig);
        let q = new_data.base.queue.clone();
        let cloned_for_bevy = new_data.clone();

        q.0.lock()
            .unwrap()
            .push(Box::new(move |w: &mut bevy::prelude::World| {
                let entity = w.spawn_empty().id();
                cloned_for_bevy.apply_to_bevy(entity, w);
            }));

        lua.create_userdata(LuaPart(new_data))?
    } else {
        return Err(mlua::Error::runtime(
            "This instance type cannot be cloned yet",
        ));
    };

    let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
    let cloned_handle = instance_handle_from_any(&cloned_ud).unwrap();
    cache.set(cloned_handle, cloned_ud.clone())?;

    if let Some(parent) = parent_for_clone {
        if let Ok(mut cloned_part) = cloned_ud.borrow_mut::<LuaPart>() {
            cloned_part.0.base.set_parent(Some(parent));
        }
    }

    let children_handles = if let Ok(part) = original_ud.borrow::<LuaPart>() {
        part.0.base.children_handles.clone()
    } else {
        Vec::new()
    };

    for child_handle in children_handles {
        if let Ok(child_ud) = cache.get::<mlua::AnyUserData>(child_handle) {
            let _ = universal_clone(lua, &child_ud, Some(cloned_ud.clone()))?;
        }
    }

    Ok(cloned_ud)
}
