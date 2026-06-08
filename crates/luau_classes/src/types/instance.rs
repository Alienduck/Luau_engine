use bevy::prelude::*;
use luau_runtime::bridge::{handle::HandleMap, queue::EngineQueue};

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
            .push(Box::new(move |w: &mut World| {
                let map = w.resource::<HandleMap>();
                let child_e = map.get_entity(handle);
                let parent_e = new_parent_handle.and_then(|h| map.get_entity(h));

                if let Some(child) = child_e {
                    if let Ok(mut e_mut) = w.get_entity_mut(child) {
                        e_mut.remove_parent_in_place();
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
    None
}
