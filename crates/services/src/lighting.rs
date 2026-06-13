use bevy::prelude::*;
use luau_classes::types::{
    color3::LuaColor3,
    instance::{CloneableInstance, InstanceData},
};
use luau_runtime::{
    bridge::{
        handle::{next_handle, HandleMap},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields};

#[derive(Component)]
pub struct LightingRoot;

#[derive(Clone)]
pub struct LuaLighting {
    pub base: InstanceData,
    pub ambient: LuaColor3,
    pub brightness: f32,
    pub global_shadows: bool,
}

impl CloneableInstance for LuaLighting {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LightingRoot);
        }
    }
}

impl UserData for LuaLighting {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });
        fields.add_field_method_get("Parent", |_, _| Ok(None::<mlua::AnyUserData>));
        fields.add_field_method_set("Parent", |_, _, _: Option<mlua::AnyUserData>| {
            Err(mlua::Error::runtime("Lighting cannot be parented"))
        });

        fields.add_field_method_get("Ambient", |_, this| Ok(this.ambient));
        fields.add_field_method_set("Ambient", |_, this, c: LuaColor3| {
            this.ambient = c;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Ok(mut ambient) = w.query::<&mut AmbientLight>().single_mut(w) {
                        ambient.color = Color::srgba(c.r, c.g, c.b, 1.0);
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Brightness", |_, this| Ok(this.brightness));
        fields.add_field_method_set("Brightness", |_, this, b: f32| {
            this.brightness = b;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Ok(mut dir_light) = w.query::<&mut DirectionalLight>().single_mut(w) {
                        dir_light.illuminance = b * 1000.0;
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("GlobalShadows", |_, this| Ok(this.global_shadows));
        fields.add_field_method_set("GlobalShadows", |_, this, s: bool| {
            this.global_shadows = s;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Ok(mut dir_light) = w.query::<&mut DirectionalLight>().single_mut(w) {
                        dir_light.shadows_enabled = s;
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        luau_classes::impl_instance_userdata!(methods);
        methods.add_method("Clone", |_, _, ()| -> mlua::Result<()> {
            Err(mlua::Error::runtime("Lighting cannot be cloned"))
        });
    }
}

pub struct LightingModule;

impl LuaModule for LightingModule {
    fn name() -> &'static str {
        "Lighting"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let handle = next_handle();
        let q = queue.clone();

        q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
            let entity = w
                .spawn((LightingRoot, Transform::default(), Visibility::Inherited))
                .id();
            w.resource_mut::<HandleMap>().insert(handle, entity, None);
        }));

        let lighting = LuaLighting {
            base: InstanceData::new(handle, queue.clone(), "Lighting"),
            ambient: LuaColor3 {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            brightness: 10.0,
            global_shadows: true,
        };

        let userdata = lua.create_userdata(lighting)?;
        lua.set_named_registry_value("__lighting_instance", userdata.clone())?;
        lua.globals().set("Lighting", userdata)?;
        Ok(())
    }
}
