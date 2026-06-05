use crate::types::{udim2::LuaUDim2, vector2::LuaVector2};
use bevy::prelude::*;
use engine_core::app::GuiObjectData;

pub struct GuiObject {
    pub anchor_point: LuaVector2,
    pub position: LuaUDim2,
    pub size: LuaUDim2,
}

impl Default for GuiObject {
    fn default() -> Self {
        Self {
            anchor_point: LuaVector2::default(),
            position: LuaUDim2::default(),
            size: LuaUDim2::default(),
        }
    }
}

pub fn apply_anchor_point_optimized(
    mut query: Query<
        (&GuiObjectData, &ComputedNode, &mut Transform),
        Or<(Changed<GuiObjectData>, Changed<ComputedNode>)>,
    >,
) {
    for (gui_data, computed_node, mut transform) in query.iter_mut() {
        let size = computed_node.size();

        transform.translation.x = -size.x * gui_data.anchor_point.x;
        transform.translation.y = -size.y * gui_data.anchor_point.y;
    }
}
