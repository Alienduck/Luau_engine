use crate::types::{
    color3::LuaColor3, gui_object::GuiObject, udim2::LuaUDim2, vector2::LuaVector2,
};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields};

pub struct LuaFrame {
    pub handle: u64,
    pub queue: EngineQueue,
    pub base: GuiObject,
    pub transparency: f32,
    pub bg_color: LuaColor3,
}

impl LuaFrame {
    fn update_layout(&self) {
        let h = self.handle;
        let s = self.base.size;
        let p = self.base.position;
        let a = self.base.anchor_point;

        let scale_x = p.x_scale - (s.x_scale * a.x);
        let offset_x = p.x_offset - (s.x_offset * a.x);
        let scale_y = p.y_scale - (s.y_scale * a.y);
        let offset_y = p.y_offset - (s.y_offset * a.y);

        self.queue
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

impl UserData for LuaFrame {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Size", |_, this| Ok(this.base.size));
        fields.add_field_method_get("Position", |_, this| Ok(this.base.position));
        fields.add_field_method_get("AnchorPoint", |_, this| Ok(this.base.anchor_point));
        fields.add_field_method_get("Transparency", |_, this| Ok(this.transparency));
        fields.add_field_method_get("BackgroundColor3", |_, this| Ok(this.bg_color));

        fields.add_field_method_set("Size", |_, this, v: LuaUDim2| {
            this.base.size = v;
            this.update_layout();
            Ok(())
        });

        fields.add_field_method_set("Position", |_, this, v: LuaUDim2| {
            this.base.position = v;
            this.update_layout();
            Ok(())
        });

        fields.add_field_method_set("AnchorPoint", |_, this, v: LuaVector2| {
            this.base.anchor_point = v;
            this.update_layout();
            Ok(())
        });

        fields.add_field_method_set("BackgroundColor3", |_, this, c: LuaColor3| {
            this.bg_color = c;
            let h = this.handle;
            let t = this.transparency;
            this.queue
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
            let h = this.handle;
            let c = this.bg_color;
            this.queue
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

        fields.add_field_method_set("Parent", |_, this, parent: mlua::AnyUserData| {
            let parent_handle =
                if let Ok(sg) = parent.borrow::<crate::instances::screen_gui::LuaScreenGui>() {
                    sg.handle
                } else if let Ok(f) = parent.borrow::<LuaFrame>() {
                    f.handle
                } else {
                    return Err(mlua::Error::runtime("Invalid parent for Frame"));
                };
            let h = this.handle;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    let map = w.resource::<HandleMap>();
                    if let (Some(child_e), Some(parent_e)) =
                        (map.get_entity(h), map.get_entity(parent_handle))
                    {
                        w.entity_mut(parent_e).add_child(child_e);
                    }
                }));
            Ok(())
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(ToString, |_, _this, ()| Ok(format!("Frame").to_owned()));
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
            lua.create_function(move |_, ()| {
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
                Ok(LuaFrame {
                    handle,
                    queue: q.clone(),
                    base: GuiObject::default(),
                    bg_color: LuaColor3 {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                    },
                    transparency: 0.0,
                })
            })?,
        )?;
        lua.globals().set("Frame", t)
    }
}
