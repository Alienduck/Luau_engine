use crate::types::{udim2::LuaUDim2, vector2::LuaVector2};

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
