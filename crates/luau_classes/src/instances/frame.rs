use crate::types::{color3::LuaColor3, udim2::LuaUDim2};
use bevy::prelude::*;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields, prelude::*};

pub struct LuaFrame {
    pub handle: u64,
    pub queue: EngineQueue,
    pub size: LuaUDim2,
    pub position: LuaUDim2,
    pub transparency: LuaValue,
    pub bg_color: LuaColor3,
}

impl UserData for LuaFrame {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Size", |_, this| Ok(this.size));
        fields.add_field_method_get("Position", |_, this| Ok(this.position));
        fields.add_field_method_get("Transparency", |_, this| Ok(this.transparency.clone()));
        fields.add_field_method_get("BackgroundColor3", |_, this| Ok(this.bg_color));
        fields.add_field_method_set("Size", |_, this, v: LuaUDim2| {
            this.size = v;
            let h = this.handle;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut n) = w.get_mut::<Node>(e) {
                            n.width = v.to_bevy_val_x();
                            n.height = v.to_bevy_val_y();
                        }
                    }
                }));
            Ok(())
        });

        fields.add_field_method_set("Position", |_, this, v: LuaUDim2| {
            this.position = v;
            let h = this.handle;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut n) = w.get_mut::<Node>(e) {
                            n.left = v.to_bevy_val_x();
                            n.top = v.to_bevy_val_y();
                        }
                    }
                }));
            Ok(())
        });

        fields.add_field_method_set("BackgroundColor3", |_, this, c: LuaColor3| {
            this.bg_color = c;
            let h = this.handle;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut bg) = w.get_mut::<BackgroundColor>(e) {
                            bg.0 = Color::srgb(c.r, c.g, c.b);
                        }
                    }
                }));
            Ok(())
        });
        fields.add_field_method_set("Transparency", |_, this, v: LuaValue| {
            let t = if let Some(t) = v.as_f32() { t } else { 0.0 };
            this.transparency = v;
            let h = this.handle;
            this.queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                        if let Some(mut bg) = w.get_mut::<BackgroundColor>(e) {
                            bg.0.set_alpha(1.0 - t);
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
                    size: LuaUDim2::default(),
                    position: LuaUDim2::default(),
                    bg_color: LuaColor3 {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                    },
                    transparency: LuaValue::Number(0.0),
                })
            })?,
        )?;
        lua.globals().set("Frame", t)
    }
}
