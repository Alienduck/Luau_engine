use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::Lua;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingStyle {
    Linear,
    Sine,
    Quad,
    Cubic,
    Quart,
    Quint,
    Bounce,
    Elastic,
    Exponential,
    Circular,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingDirection {
    In,
    Out,
    InOut,
}

#[derive(Clone, Copy, Debug)]
pub enum RenderCollisionMode {
    ColliderShapes,
    RigidbodyAxes,
    MultiBodyJoints,
    ImpulseJoints,
    Joints,
    SolverContacts,
    Contacts,
    ColliderAABBS,
}

pub struct EnumsModule;
impl LuaModule for EnumsModule {
    fn name() -> &'static str {
        "Enums"
    }
    fn register(lua: &Lua, _: &EngineQueue) -> mlua::Result<()> {
        let env = lua.globals();
        let enum_table = env
            .get::<mlua::Table>("Enum")
            .unwrap_or_else(|_| lua.create_table().unwrap());

        let part_type = lua.create_table()?;
        part_type.set("Block", 0)?;
        part_type.set("Ball", 1)?;
        part_type.set("Cylinder", 2)?;
        enum_table.set("PartType", part_type)?;

        let col_fid = lua.create_table()?;
        col_fid.set("Default", 0)?;
        col_fid.set("Hull", 1)?;
        col_fid.set("Box", 2)?;
        col_fid.set("PreciseConvexDecomposition", 3)?;
        enum_table.set("CollisionFidelity", col_fid)?;

        let easing_style = lua.create_table()?;
        easing_style.set("Linear", EasingStyle::Linear as u8)?;
        easing_style.set("Sine", EasingStyle::Sine as u8)?;
        easing_style.set("Quad", EasingStyle::Quad as u8)?;
        easing_style.set("Cubic", EasingStyle::Cubic as u8)?;
        easing_style.set("Quart", EasingStyle::Quart as u8)?;
        easing_style.set("Quint", EasingStyle::Quint as u8)?;
        easing_style.set("Bounce", EasingStyle::Bounce as u8)?;
        easing_style.set("Elastic", EasingStyle::Elastic as u8)?;
        easing_style.set("Exponential", EasingStyle::Exponential as u8)?;
        easing_style.set("Circular", EasingStyle::Circular as u8)?;
        easing_style.set("Back", EasingStyle::Back as u8)?;

        let easing_direction = lua.create_table()?;
        easing_direction.set("In", EasingDirection::In as u8)?;
        easing_direction.set("Out", EasingDirection::Out as u8)?;
        easing_direction.set("InOut", EasingDirection::InOut as u8)?;

        enum_table.set("EasingStyle", easing_style)?;
        enum_table.set("EasingDirection", easing_direction)?;

        let debug_render_mode = lua.create_table()?;
        debug_render_mode.set("ColliderShapes", 0_u8)?;
        debug_render_mode.set("RigidbodyAxes", 1_u8)?;
        debug_render_mode.set("MultiBodyJoints", 2_u8)?;
        debug_render_mode.set("ImpulseJoints", 3_u8)?;
        debug_render_mode.set("Joints", 4_u8)?;
        debug_render_mode.set("SolverContacts", 5_u8)?;
        debug_render_mode.set("Contacts", 6_u8)?;
        debug_render_mode.set("ColliderAABBS", 7_u8)?;
        enum_table.set("ColliderRenderMode", debug_render_mode)?;

        env.set("Enum", enum_table)?;
        Ok(())
    }
}
