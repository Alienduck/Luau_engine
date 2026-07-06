use crate::{
    impl_base_instance_fields, impl_gui_object_fields,
    instances::ui::ui_interactions::{LuauButtonSignals, PreviousInteraction},
    types::{
        color3::LuaColor3,
        gui_object::GuiObject,
        instance::{CloneableInstance, InstanceData},
        signal::LuaSignal,
    },
};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

#[derive(Clone)]
pub struct LuaTextButton {
    pub base: InstanceData,
    pub gui: GuiObject,
    pub text: String,
    pub text_color: LuaColor3,
    pub text_size: f32,
    pub click_id: u64,
    pub enter_id: u64,
    pub leave_id: u64,
}

impl CloneableInstance for LuaTextButton {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    /// Automaticly generate new signales on clone
    fn on_cloned(&mut self, lua: &Lua) -> mlua::Result<()> {
        self.click_id = LuaSignal::new(lua)?.id;
        self.enter_id = LuaSignal::new(lua)?.id;
        self.leave_id = LuaSignal::new(lua)?.id;
        Ok(())
    }

    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert((
                Button,
                Interaction::None,
                PreviousInteraction::default(),
                LuauButtonSignals {
                    click: self.click_id,
                    enter: self.enter_id,
                    leave: self.leave_id,
                },
            ));
        }
    }
}

impl UserData for LuaTextButton {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        impl_base_instance_fields!(fields);
        impl_gui_object_fields!(fields);

        fields.add_field_method_get("MouseButton1Click", |_, this| {
            Ok(LuaSignal { id: this.click_id })
        });
        fields.add_field_method_get("MouseEnter", |_, this| Ok(LuaSignal { id: this.enter_id }));
        fields.add_field_method_get("MouseLeave", |_, this| Ok(LuaSignal { id: this.leave_id }));

        fields.add_field_method_get("Text", |_, this| Ok(this.text.clone()));
        fields.add_field_method_set("Text", |_, this, v: String| {
            this.text = v.clone();
            let h = this.base.handle;
            this.base
                .queue
                .push(luau_runtime::bridge::queue::EngineCommand::SetText { handle: h, text: v });
            Ok(())
        });

        fields.add_field_method_get("TextColor3", |_, this| Ok(this.text_color));
        fields.add_field_method_set("TextColor3", |_, this, c: LuaColor3| {
            this.text_color = c;
            let h = this.base.handle;
            this.base
                .queue
                .push(luau_runtime::bridge::queue::EngineCommand::SetTextColor {
                    handle: h,
                    r: c.r,
                    g: c.g,
                    b: c.b,
                });
            Ok(())
        });

        fields.add_field_method_get("TextSize", |_, this| Ok(this.text_size));
        fields.add_field_method_set("TextSize", |_, this, s: f32| {
            this.text_size = s;
            let h = this.base.handle;
            this.base
                .queue
                .push(luau_runtime::bridge::queue::EngineCommand::SetFontSize {
                    handle: h,
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

pub struct TextButtonModule;

impl LuaModule for TextButtonModule {
    fn name() -> &'static str {
        "TextButton"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let button = LuaTextButton {
                    base: InstanceData::new(handle, q.clone(), "TextButton"),
                    gui: GuiObject::default(),
                    text: "Button".to_string(),
                    text_color: LuaColor3 {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    },
                    text_size: 14.0,
                    click_id: LuaSignal::new(lua_ctx)?.id,
                    enter_id: LuaSignal::new(lua_ctx)?.id,
                    leave_id: LuaSignal::new(lua_ctx)?.id,
                };

                let clone_for_spawn = button.clone();
                q.push_raw(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                            Text::new("Button"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.0, 0.0, 0.0)),
                        ))
                        .id();
                    clone_for_spawn.apply_bevy_components(entity, w);
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                });

                let ud = lua_ctx.create_userdata(button)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("TextButton", t)
    }
}
