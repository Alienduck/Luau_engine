use mlua::Lua;

/// Wraps the Luau VM so it can be stored as a Bevy non-send resource.
pub struct LuaVm(pub Lua);

impl LuaVm {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        inject_task_stdlib(&lua)?;
        Ok(Self(lua))
    }

    pub fn lua(&self) -> &Lua {
        &self.0
    }
}

impl Default for LuaVm {
    fn default() -> Self {
        Self::new().expect("failed to create Lua VM")
    }
}

/// Inject `task.*` helpers that mirror the Roblox task library.
fn inject_task_stdlib(lua: &Lua) -> mlua::Result<()> {
    lua.load(
        r#"
        task = task or {}

        function task.wait(t)
            coroutine.yield("wait", t or 0)
        end

        function task.spawn(f, ...)
            local t = coroutine.create(f)
            coroutine.resume(t, ...)
            return t
        end

        function task.defer(f, ...)
            return task.spawn(function(...)
                task.wait(0)
                f(...)
            end, ...)
        end
    "#,
    )
    .set_name("task_stdlib")
    .exec()
}
