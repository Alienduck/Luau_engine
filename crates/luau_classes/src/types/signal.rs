use mlua::{Lua, UserData, UserDataMethods};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct LuaConnection {
    pub signal_id: u64,
    pub conn_id: u64,
}

impl UserData for LuaConnection {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Disconnect", |lua, this, ()| {
            if let Ok(signals) = lua.named_registry_value::<mlua::Table>("__signals") {
                if let Ok(callbacks) = signals.get::<mlua::Table>(this.signal_id) {
                    let _ = callbacks.set(this.conn_id, mlua::Value::Nil);
                }
            }
            Ok(())
        });
    }
}

#[derive(Clone)]
pub struct LuaSignal {
    pub id: u64,
}

impl LuaSignal {
    pub fn new(lua: &Lua) -> mlua::Result<Self> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let signals: mlua::Table = match lua.named_registry_value("__signals") {
            Ok(t) => t,
            Err(_) => {
                let t = lua.create_table()?;
                lua.set_named_registry_value("__signals", t.clone())?;
                t
            }
        };
        signals.set(id, lua.create_table()?)?;
        Ok(Self { id })
    }

    pub fn fire<'lua, A: mlua::IntoLuaMulti>(&self, lua: &'lua Lua, args: A) -> mlua::Result<()> {
        let multi_args = args.into_lua_multi(lua)?;
        if let Ok(signals) = lua.named_registry_value::<mlua::Table>("__signals") {
            if let Ok(callbacks) = signals.get::<mlua::Table>(self.id) {
                let task_spawn: Option<mlua::Function> = lua.globals().get("__task_spawn").ok();

                for pair in callbacks.pairs::<u64, mlua::Function>() {
                    if let Ok((_, func)) = pair {
                        if let Some(ref spawner) = task_spawn {
                            if let Err(e) = spawner.call::<()>((func, multi_args.clone())) {
                                bevy::log::error!("[Luau Error] {}", e);
                            }
                        } else {
                            if let Err(e) = func.call::<()>(multi_args.clone()) {
                                bevy::log::error!("[Luau Error] {}", e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl UserData for LuaSignal {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Connect", |lua, this, callback: mlua::Function| {
            let conn_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let signals: mlua::Table = lua.named_registry_value("__signals")?;
            let callbacks: mlua::Table = signals.get(this.id)?;
            callbacks.set(conn_id, callback)?;
            Ok(LuaConnection {
                signal_id: this.id,
                conn_id,
            })
        });
    }
}
