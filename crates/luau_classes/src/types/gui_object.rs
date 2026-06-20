use bevy::ui::Val;
use luau_runtime::bridge::queue::{EngineCommand, EngineQueue};

use crate::types::{color3::LuaColor3, udim2::LuaUDim2, vector2::LuaVector2};

#[derive(Clone)]
pub struct GuiObject {
    pub anchor_point: LuaVector2,
    pub position: LuaUDim2,
    pub size: LuaUDim2,
    pub background_transparency: f32,
    pub background_color: LuaColor3,
    pub visible: bool,
}

impl Default for GuiObject {
    fn default() -> Self {
        Self {
            anchor_point: LuaVector2::default(),
            position: LuaUDim2::default(),
            size: LuaUDim2::default(),
            background_color: LuaColor3::default(),
            background_transparency: 0.0,
            visible: true,
        }
    }
}

impl GuiObject {
    /// Pushes a single typed [`EngineCommand::SetNodeLayout`] command.
    /// No closure, no Box, no heap allocation.
    pub fn enqueue_layout_update(&self, handle: u64, queue: &EngineQueue) {
        let s = self.size;
        let p = self.position;
        let a = self.anchor_point;

        let scale_x = p.x_scale - s.x_scale * a.x;
        let offset_x = p.x_offset - s.x_offset * a.x;
        let scale_y = p.y_scale - s.y_scale * a.y;
        let offset_y = p.y_offset - s.y_offset * a.y;

        let width = if s.x_scale != 0.0 {
            Val::Percent(s.x_scale * 100.0)
        } else {
            Val::Px(s.x_offset)
        };
        let height = if s.y_scale != 0.0 {
            Val::Percent(s.y_scale * 100.0)
        } else {
            Val::Px(s.y_offset)
        };

        queue.push(EngineCommand::SetNodeLayout {
            handle,
            left: Val::Percent(scale_x * 100.0),
            top: Val::Percent(scale_y * 100.0),
            width,
            height,
            margin_left: Val::Px(offset_x),
            margin_top: Val::Px(offset_y),
        });

        queue.push(EngineCommand::SetVisibility {
            handle,
            visible: self.visible,
        });
    }

    /// Pushes a single typed [`EngineCommand::SetBackgroundColor`] command.
    pub fn enqueue_color_update(&self, handle: u64, queue: &EngineQueue) {
        let c = self.background_color;
        let a = 1.0 - self.background_transparency;
        queue.push(EngineCommand::SetBackgroundColor {
            handle,
            r: c.r,
            g: c.g,
            b: c.b,
            a,
        });
    }
}

#[macro_export]
macro_rules! impl_gui_object_fields {
    ($fields:ident) => {
        $fields.add_field_method_get("Size", |_, this| Ok(this.gui.size));
        $fields.add_field_method_set("Size", |_, this, v: $crate::types::udim2::LuaUDim2| {
            use $crate::types::instance::CloneableInstance;
            this.gui.size = v;
            this.gui
                .enqueue_layout_update(this.base().handle, &this.base().queue);
            Ok(())
        });

        $fields.add_field_method_get("Position", |_, this| Ok(this.gui.position));
        $fields.add_field_method_set("Position", |_, this, v: $crate::types::udim2::LuaUDim2| {
            use $crate::types::instance::CloneableInstance;
            this.gui.position = v;
            this.gui
                .enqueue_layout_update(this.base().handle, &this.base().queue);
            Ok(())
        });

        $fields.add_field_method_get("AnchorPoint", |_, this| Ok(this.gui.anchor_point));
        $fields.add_field_method_set(
            "AnchorPoint",
            |_, this, v: $crate::types::vector2::LuaVector2| {
                use $crate::types::instance::CloneableInstance;
                this.gui.anchor_point = v;
                this.gui
                    .enqueue_layout_update(this.base().handle, &this.base().queue);
                Ok(())
            },
        );

        $fields.add_field_method_get("BackgroundColor3", |_, this| Ok(this.gui.background_color));
        $fields.add_field_method_set(
            "BackgroundColor3",
            |_, this, v: $crate::types::color3::LuaColor3| {
                use $crate::types::instance::CloneableInstance;
                this.gui.background_color = v;
                this.gui
                    .enqueue_color_update(this.base().handle, &this.base().queue);
                Ok(())
            },
        );

        $fields.add_field_method_get("BackgroundTransparency", |_, this| {
            Ok(this.gui.background_transparency)
        });
        $fields.add_field_method_set("BackgroundTransparency", |_, this, v: f32| {
            use $crate::types::instance::CloneableInstance;
            this.gui.background_transparency = v;
            this.gui
                .enqueue_color_update(this.base().handle, &this.base().queue);
            Ok(())
        });
        $fields.add_field_method_get("Visible", |_, this| Ok(this.gui.visible));
        $fields.add_field_method_set("Visible", |_, this, v: bool| {
            use $crate::types::instance::CloneableInstance;
            this.gui.visible = v;
            this.gui
                .enqueue_layout_update(this.base().handle, &this.base().queue);
            Ok(())
        })
    };
}
