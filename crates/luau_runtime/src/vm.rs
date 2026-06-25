use crate::scheduler::SpawnerQueue;
use mlua::{Lua, Value};
use std::{cell::RefCell, fs, path::Path, rc::Rc};

pub struct LuaVm {
    pub lua: Lua,
    pub spawner_queue: SpawnerQueue,
}

impl LuaVm {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        let spawner_queue = SpawnerQueue(Rc::new(RefCell::new(Vec::new())));

        inject_task_stdlib(&lua, spawner_queue.clone())?;
        inject_module_system(&lua)?;

        let cache = lua.create_table()?;

        lua.set_named_registry_value("__instance_cache", cache)?;

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

fn inject_module_system(lua: &Lua) -> mlua::Result<()> {
    lua.set_named_registry_value("__loaded_modules", lua.create_table()?)?;

    let require_func = lua.create_function(|lua, path_str: String| {
        let mut resolved_path_str = path_str.clone();

        if let Ok(mut caller_source) = lua.load("return debug.info(3, 's')").eval::<String>() {
            if caller_source.starts_with('@') || caller_source.starts_with('=') {
                caller_source = caller_source[1..].to_string();
            } else if caller_source.starts_with("[string \"") && caller_source.ends_with("\"]") {
                caller_source = caller_source[9..caller_source.len() - 2].to_string();
            }

            let caller_path = Path::new(&caller_source);

            if let Some(parent_dir) = caller_path.parent() {
                if parent_dir.as_os_str() != "" {
                    let joined_path = parent_dir.join(&path_str);
                    resolved_path_str = joined_path.to_string_lossy().to_string();
                }
            }
        }

        if !resolved_path_str.ends_with(".luau") && !resolved_path_str.ends_with(".lua") {
            resolved_path_str.push_str(".luau");
        }

        let path = Path::new(&resolved_path_str);
        if !path.exists() {
            return Err(mlua::Error::runtime(format!(
                "Module not found: {} (resolved to: {})",
                path_str, resolved_path_str
            )));
        }

        let loaded_table: mlua::Table = lua.named_registry_value("__loaded_modules")?;

        if let Ok(cached_value) = loaded_table.get::<Value>(resolved_path_str.as_str()) {
            if !matches!(cached_value, Value::Nil) {
                return Ok(cached_value);
            }
        }

        let source = fs::read_to_string(path).map_err(|e| {
            mlua::Error::runtime(format!(
                "Failed to read module '{}': {}",
                resolved_path_str, e
            ))
        })?;

        let chunk_name = format!("@{}", resolved_path_str);
        let module_chunk = lua.load(&source).set_name(&chunk_name).into_function()?;

        let result: Value = module_chunk.call(())?;

        let cache_value = if matches!(result, Value::Nil) {
            Value::Boolean(true)
        } else {
            result.clone()
        };

        loaded_table.set(resolved_path_str.as_str(), cache_value)?;

        Ok(result)
    })?;

    lua.globals().set("require", require_func)?;
    Ok(())
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
