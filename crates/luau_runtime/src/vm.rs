use crate::scheduler::SpawnerQueue;
use mlua::Lua;
use std::{cell::RefCell, rc::Rc};

pub struct LuaVm {
    pub lua: Lua,
    pub spawner_queue: SpawnerQueue,
}

impl LuaVm {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        let spawner_queue = SpawnerQueue(Rc::new(RefCell::new(Vec::new())));
        inject_task_stdlib(&lua, spawner_queue.clone())?;
        Ok(Self { lua, spawner_queue })
    }
    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}

impl Default for LuaVm {
    fn default() -> Self {
        Self::new().expect("failed to create Lua VM")
    }
}

fn inject_task_stdlib(lua: &Lua, spawner: SpawnerQueue) -> mlua::Result<()> {
    let spawn_spawner = spawner.clone();
    let spawn_func =
        lua.create_function(move |lua, (func, args): (mlua::Value, mlua::MultiValue)| {
            let thread = match func {
                mlua::Value::Function(f) => lua.create_thread(f)?,
                mlua::Value::Thread(t) => t,
                _ => return Err(mlua::Error::runtime("expected function or thread")),
            };

            let result: mlua::MultiValue = thread.resume(args)?;

            if thread.status() == mlua::ThreadStatus::Resumable {
                let mut iter = result.into_iter();
                if let Some(mlua::Value::String(s)) = iter.next() {
                    if s.to_str().unwrap() == "wait" {
                        let delay = iter
                            .next()
                            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
                            .unwrap_or(0.0);
                        spawn_spawner.0.borrow_mut().push((thread.clone(), delay));
                        return Ok(thread);
                    }
                }
                spawn_spawner.0.borrow_mut().push((thread.clone(), 0.0));
            }
            Ok(thread)
        })?;
    lua.globals().set("__task_spawn", spawn_func)?;

    let defer_spawner = spawner;
    let defer_func = lua.create_function(move |lua, func: mlua::Value| {
        let thread = match func {
            mlua::Value::Function(f) => lua.create_thread(f)?,
            mlua::Value::Thread(t) => t,
            _ => return Err(mlua::Error::runtime("expected function or thread")),
        };
        defer_spawner.0.borrow_mut().push((thread.clone(), 0.0));
        Ok(thread)
    })?;
    lua.globals().set("__task_defer", defer_func)?;

    lua.load(
        r#"
        task = task or {}
        function task.wait(t)
            return coroutine.yield("wait", t or 0)
        end
        function task.spawn(f, ...)
            return __task_spawn(f, ...)
        end
        function task.defer(f, ...)
            local args = {...}
            if #args > 0 then
                return __task_defer(coroutine.create(function() f(unpack(args)) end))
            else
                return __task_defer(f)
            end
        end
        "#,
    )
    .set_name("task_stdlib")
    .exec()
}
