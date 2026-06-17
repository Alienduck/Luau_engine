use crate::types::{
    color3::LuaColor3,
    instance::{CloneableInstance, InstanceData},
};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};

#[derive(Component, Clone)]
pub struct LuauAtmosphere {
    pub density: f32,
    pub color: LuaColor3,
    pub decay: LuaColor3,
    pub glare: f32,
    pub haze: f32,
}

#[derive(Clone)]
pub struct LuaAtmosphere {
    pub base: InstanceData,
    pub density: f32,
    pub color: LuaColor3,
    pub decay: LuaColor3,
    pub glare: f32,
    pub haze: f32,
}

impl CloneableInstance for LuaAtmosphere {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LuauAtmosphere {
                density: self.density,
                color: self.color,
                decay: self.decay,
                glare: self.glare,
                haze: self.haze,
            });
        }
    }
}

impl UserData for LuaAtmosphere {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });

        fields.add_field_method_get("Parent", |lua, this| {
            if let Some(parent_handle) = this.base.parent_handle {
                let instances_map: mlua::Table = lua.named_registry_value("__instance_cache")?;
                Ok(instances_map.get::<mlua::AnyUserData>(parent_handle).ok())
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            this.base.set_parent(lua, parent);
            Ok(())
        });

        fields.add_field_method_get("Density", |_, this| Ok(this.density));
        fields.add_field_method_set("Density", |_, this, v: f32| {
            this.density = v;
            let h = this.base.handle;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut a) = w.get_mut::<LuauAtmosphere>(e) {
                            a.density = v;
                        }
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Color", |_, this| Ok(this.color));
        fields.add_field_method_set("Color", |_, this, v: LuaColor3| {
            this.color = v;
            let h = this.base.handle;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut a) = w.get_mut::<LuauAtmosphere>(e) {
                            a.color = v;
                        }
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Decay", |_, this| Ok(this.decay));
        fields.add_field_method_set("Decay", |_, this, v: LuaColor3| {
            this.decay = v;
            let h = this.base.handle;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut a) = w.get_mut::<LuauAtmosphere>(e) {
                            a.decay = v;
                        }
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Glare", |_, this| Ok(this.glare));
        fields.add_field_method_set("Glare", |_, this, v: f32| {
            this.glare = v;
            let h = this.base.handle;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut a) = w.get_mut::<LuauAtmosphere>(e) {
                            a.glare = v;
                        }
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Haze", |_, this| Ok(this.haze));
        fields.add_field_method_set("Haze", |_, this, v: f32| {
            this.haze = v;
            let h = this.base.handle;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut a) = w.get_mut::<LuauAtmosphere>(e) {
                            a.haze = v;
                        }
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
    }
}

pub struct AtmosphereModule;

impl LuaModule for AtmosphereModule {
    fn name() -> &'static str {
        "Atmosphere"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_cache, ()| {
                let handle = next_handle();
                let effect = LuaAtmosphere {
                    base: InstanceData::new(handle, q.clone(), "Atmosphere"),
                    density: 0.3,
                    color: LuaColor3 {
                        r: 0.78,
                        g: 0.78,
                        b: 0.78,
                    },
                    decay: LuaColor3 {
                        r: 0.41,
                        g: 0.44,
                        b: 0.49,
                    },
                    glare: 0.0,
                    haze: 0.0,
                };

                let clone_for_spawn = effect.clone();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = clone_for_spawn.base().spawn_base_entity(w);
                    clone_for_spawn.apply_bevy_components(entity, w);
                }));

                let userdata = lua_cache.create_userdata(effect)?;
                if let Ok(cache) = lua_cache.named_registry_value::<mlua::Table>("__instance_cache")
                {
                    let _ = cache.set(handle, userdata.clone());
                }
                Ok(userdata)
            })?,
        )?;
        lua.globals().set("Atmosphere", t)
    }
}
