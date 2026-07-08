use luau_runtime::{bridge::queue::EngineQueue, registry::LuaModule};
use mlua::{Lua, UserData, UserDataMethods};

#[derive(Clone)]
pub struct PhysicsService {
    queue: EngineQueue,
}

impl UserData for PhysicsService {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("RegisterCollisionGroup", |_, this, name: String| {
            this.queue
                .push(luau_runtime::bridge::queue::EngineCommand::RegisterCollisionGroup { name });
            Ok(())
        });
        methods.add_method(
            "CollisionGroupSetCollidable",
            |_, this, (group1, group2, collidable): (String, String, bool)| {
                this.queue.push(
                    luau_runtime::bridge::queue::EngineCommand::SetCollisionGroupCollidable {
                        group1,
                        group2,
                        collidable,
                    },
                );
                Ok(())
            },
        );
    }
}

pub struct PhysicsServiceModule;

impl LuaModule for PhysicsServiceModule {
    fn name() -> &'static str {
        "PhysicsService"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let service = PhysicsService {
            queue: queue.clone(),
        };
        lua.globals().set("PhysicsService", service)?;
        Ok(())
    }
}
