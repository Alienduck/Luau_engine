use crate::types::{
    gui_object::GuiObject,
    instance::{CloneableInstance, InstanceData},
};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::{EngineCommand, EngineQueue},
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

#[derive(Clone)]
pub struct LuaImageLabel {
    pub base: InstanceData,
    pub gui: GuiObject,
    pub image: String,
}

impl CloneableInstance for LuaImageLabel {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {}
}

impl UserData for LuaImageLabel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);
        crate::impl_gui_object_fields!(fields);

        fields.add_field_method_get("Image", |_, this| Ok(this.image.clone()));
        fields.add_field_method_set("Image", |_, this, v: String| {
            this.image = v.clone();
            let h = this.base.handle;
            this.base.queue.push(EngineCommand::SetImageNode {
                handle: h,
                asset_path: v,
            });
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base().name.clone()));
    }
}

pub struct ImageLabelModule;

impl LuaModule for ImageLabelModule {
    fn name() -> &'static str {
        "ImageLabel"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                q.push_raw(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                        ))
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                });

                let label = LuaImageLabel {
                    base: InstanceData::new(handle, q.clone(), "ImageLabel"),
                    gui: GuiObject::default(),
                    image: "".to_string(),
                };

                let ud = lua_ctx.create_userdata(label)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("ImageLabel", t)
    }
}
