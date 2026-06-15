use crate::types::{
    color3::LuaColor3,
    gui_object::GuiObject,
    instance::{CloneableInstance, InstanceData},
    udim2::LuaUDim2,
    vector2::LuaVector2,
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

/// Luau-facing `Frame` — a rectangular 2-D UI element.
///
/// Position and size are expressed as [`LuaUDim2`] values (scale + offset),
/// following the Roblox convention.  The frame maps to a Bevy [`Node`] with
/// `PositionType::Absolute`.
#[derive(Clone)]
pub struct LuaFrame {
    pub base: InstanceData,
    pub gui: GuiObject,
    pub transparency: f32,
    pub bg_color: LuaColor3,
}

impl LuaFrame {
    /// Recomputes the Bevy node layout from the current `position`, `size`,
    /// and `anchor_point` and enqueues the update.
    fn update_layout(&self) {
        let h = self.base.handle;
        let s = self.gui.size;
        let p = self.gui.position;
        let a = self.gui.anchor_point;

        let scale_x = p.x_scale - s.x_scale * a.x;
        let offset_x = p.x_offset - s.x_offset * a.x;
        let scale_y = p.y_scale - s.y_scale * a.y;
        let offset_y = p.y_offset - s.y_offset * a.y;

        self.base
            .queue
            .0
            .lock()
            .unwrap()
            .push(Box::new(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    if let Some(mut n) = w.get_mut::<Node>(e) {
                        n.width = if s.x_scale != 0.0 {
                            Val::Percent(s.x_scale * 100.0)
                        } else {
                            Val::Px(s.x_offset)
                        };
                        n.height = if s.y_scale != 0.0 {
                            Val::Percent(s.y_scale * 100.0)
                        } else {
                            Val::Px(s.y_offset)
                        };
                        n.left = Val::Percent(scale_x * 100.0);
                        n.margin.left = Val::Px(offset_x);
                        n.top = Val::Percent(scale_y * 100.0);
                        n.margin.top = Val::Px(offset_y);
                    }
                }
            }));
    }
}

impl CloneableInstance for LuaFrame {
    fn base(&self) -> &InstanceData {
        &self.base
    }

    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }

    fn apply_bevy_components(&self, _entity: Entity, _w: &mut World) {
        // Layout is applied lazily via `update_layout` when properties are set.
    }
}

impl UserData for LuaFrame {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_get("ClassName", |_, this| Ok(this.base.class_name));
        fields.add_field_method_get("Size", |_, this| Ok(this.gui.size));
        fields.add_field_method_get("Position", |_, this| Ok(this.gui.position));
        fields.add_field_method_get("AnchorPoint", |_, this| Ok(this.gui.anchor_point));
        fields.add_field_method_get("Transparency", |_, this| Ok(this.transparency));
        fields.add_field_method_get("BackgroundColor3", |_, this| Ok(this.bg_color));
        fields.add_field_method_get("Parent", |lua, this| {
            let Some(parent_handle) = this.base.parent_handle else {
                return Ok(None);
            };
            let cache: mlua::Table = lua.named_registry_value("__instance_cache")?;
            Ok(cache.get::<Option<mlua::AnyUserData>>(parent_handle)?)
        });

        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });
        fields.add_field_method_set("Size", |_, this, v: LuaUDim2| {
            this.gui.size = v;
            this.update_layout();
            Ok(())
        });
        fields.add_field_method_set("Position", |_, this, v: LuaUDim2| {
            this.gui.position = v;
            this.update_layout();
            Ok(())
        });
        fields.add_field_method_set("AnchorPoint", |_, this, v: LuaVector2| {
            this.gui.anchor_point = v;
            this.update_layout();
            Ok(())
        });
        fields.add_field_method_set("BackgroundColor3", |_, this, c: LuaColor3| {
            this.bg_color = c;
            let h = this.base.handle;
            let t = this.transparency;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut bg) = w.get_mut::<BackgroundColor>(e) {
                            bg.0 = Color::srgba(c.r, c.g, c.b, 1.0 - t);
                        }
                    }
                }));
            Ok(())
        });
        fields.add_field_method_set("Transparency", |_, this, t: f32| {
            this.transparency = t;
            let h = this.base.handle;
            let c = this.bg_color;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut bg) = w.get_mut::<BackgroundColor>(e) {
                            bg.0 = Color::srgba(c.r, c.g, c.b, 1.0 - t);
                        }
                    }
                }));
            Ok(())
        });
        // Parent accepts ScreenGui or Frame.
        fields.add_field_method_set("Parent", |lua, this, parent: Option<mlua::AnyUserData>| {
            this.base.set_parent(lua, parent);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
    }
}

pub struct FrameModule;

impl LuaModule for FrameModule {
    fn name() -> &'static str {
        "Frame"
    }

    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;
        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
                    let entity = w
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                        ))
                        .id();
                    w.resource_mut::<HandleMap>().insert(handle, entity, None);
                }));

                let frame = LuaFrame {
                    base: InstanceData::new(handle, q.clone(), "Frame"),
                    gui: GuiObject::default(),
                    bg_color: LuaColor3 {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                    },
                    transparency: 0.0,
                };
                let ud = lua_ctx.create_userdata(frame)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;
        lua.globals().set("Frame", t)
    }
}
