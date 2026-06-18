use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::Lua;

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

        env.set("Enum", enum_table)?;
        Ok(())
    }
}
