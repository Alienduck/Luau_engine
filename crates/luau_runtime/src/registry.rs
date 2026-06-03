use crate::bridge::queue::EngineQueue;
use mlua::Lua;

/// Every Luau-facing module (class or service) implements this trait.
/// The engine calls `register` once at startup to inject globals into the VM.
pub trait LuaModule {
    fn name() -> &'static str;
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()>;
}
