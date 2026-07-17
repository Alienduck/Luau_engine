use crate::types::{
    color3::LuaColor3,
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
pub struct LuaTextLabel {
    pub base: InstanceData,
    pub gui: GuiObject,
    pub text: String,
    pub text_color: LuaColor3,
    pub text_size: f32,
}

impl CloneableInstance for LuaTextLabel {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {}
}

impl UserData for LuaTextLabel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);
        crate::impl_gui_object_fields!(fields);

        fields.add_field_method_get("Text", |_, this| Ok(this.text.clone()));
        fields.add_field_method_set("Text", |_, this, v: String| {
            this.text = v.clone();
            this.base.queue.push(EngineCommand::SetText {
                handle: this.base.handle,
                text: v,
            });
            Ok(())
        });

        fields.add_field_method_get("TextColor3", |_, this| Ok(this.text_color));
        fields.add_field_method_set("TextColor3", |_, this, c: LuaColor3| {
            this.text_color = c;
            this.base.queue.push(EngineCommand::SetTextColor {
                handle: this.base.handle,
                r: c.r,
                g: c.g,
                b: c.b,
            });
            Ok(())
        });

        fields.add_field_method_get("TextSize", |_, this| Ok(this.text_size));
        fields.add_field_method_set("TextSize", |_, this, s: f32| {
            this.text_size = s;
            this.base.queue.push(EngineCommand::SetFontSize {
                handle: this.base.handle,
                size: s,
            });
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base().name.clone()));
    }
}

pub struct TextLabelModule;

impl LuaModule for TextLabelModule {
    fn name() -> &'static str {
        "TextLabel"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let destroying_signal_id = crate::types::signal::LuaSignal::new(lua_ctx)?.id;
                q.push_raw(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                            Text::new("TextLabel"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.0, 0.0, 0.0)),
                        ))
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                });

                let label = LuaTextLabel {
                    base: InstanceData::new(handle, q.clone(), "TextLabel", destroying_signal_id),
                    gui: GuiObject::default(),
                    text: "TextLabel".to_string(),
                    text_color: LuaColor3 {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    },
                    text_size: 14.0,
                };

                let ud = lua_ctx.create_userdata(label)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("TextLabel", t)
    }
}
