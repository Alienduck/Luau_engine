use crate::instances::base_part::BasePartData;
use crate::instances::mesh_part::{LuaMeshPart, LuauCollisionFidelity};
use crate::types::instance::{CloneableInstance, InstanceData};
use crate::types::signal::LuaSignal;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::scene::{SceneInstance, SceneSpawner};
use luau_runtime::vm::LuaVm;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::ObjectLike;
use mlua::{Lua, MetaMethod::ToString, UserData, UserDataFields, UserDataMethods};

#[derive(Component)]
pub struct PendingModelScene {
    pub loaded_signal_id: u64,
    pub queue: EngineQueue,
}

#[derive(Clone)]
pub struct LuaModel {
    pub base: InstanceData,
    pub collision_fidelity: u8,
    pub loaded_signal_id: u64,
}

impl CloneableInstance for LuaModel {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        let comp = match self.collision_fidelity {
            1 => LuauCollisionFidelity::Hull,
            2 => LuauCollisionFidelity::Box,
            3 => LuauCollisionFidelity::Precise,
            _ => LuauCollisionFidelity::Default,
        };
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(comp);
        }
    }
}

impl UserData for LuaModel {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        crate::impl_base_instance_fields!(fields);

        fields.add_field_method_get("Loaded", |_, this| {
            Ok(LuaSignal {
                id: this.loaded_signal_id,
            })
        });

        fields.add_field_method_get("CollisionFidelity", |_, this| Ok(this.collision_fidelity));
        fields.add_field_method_set("CollisionFidelity", |_, this, v: u8| {
            this.collision_fidelity = v;
            let h = this.base.handle;
            this.base.queue.push_raw(move |w: &mut World| {
                if let Some(e) = w.resource::<HandleMap>().get_entity(h) {
                    let comp = match v {
                        1 => LuauCollisionFidelity::Hull,
                        2 => LuauCollisionFidelity::Box,
                        3 => LuauCollisionFidelity::Precise,
                        _ => LuauCollisionFidelity::Default,
                    };
                    if let Ok(mut em) = w.get_entity_mut(e) {
                        em.insert(comp);
                    }
                }
            });
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        crate::impl_instance_userdata!(methods);
        methods.add_meta_method(ToString, |_, this, ()| Ok(this.base.name.clone()));
    }
}

pub struct ModelModule;

impl LuaModule for ModelModule {
    fn name() -> &'static str {
        "Model"
    }

    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let q = queue.clone();
        let t = lua.create_table()?;

        t.set(
            "new",
            lua.create_function(move |lua_ctx, ()| {
                let handle = next_handle();
                let model = LuaModel {
                    base: InstanceData::new(handle, q.clone(), "Model"),
                    collision_fidelity: 0,
                    loaded_signal_id: 0,
                };

                let spawn_copy = model.clone();
                q.push_raw(move |w: &mut World| {
                    let entity = spawn_copy.base().spawn_base_entity(w);
                    spawn_copy.apply_bevy_components(entity, w);
                });

                let ud = lua_ctx.create_userdata(model)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;

        let q_from = queue.clone();
        t.set(
            "from",
            lua.create_function(move |lua_ctx, path: String| {
                let handle = next_handle();
                let loaded_signal = LuaSignal::new(lua_ctx)?;

                let model = LuaModel {
                    base: InstanceData::new(handle, q_from.clone(), "Model"),
                    collision_fidelity: 0,
                    loaded_signal_id: loaded_signal.id,
                };

                let spawn_copy = model.clone();
                let asset_path = path.clone();
                let q_for_comp = q_from.clone();

                q_from.push_raw(move |w: &mut World| {
                    let entity = spawn_copy.base().spawn_base_entity(w);
                    spawn_copy.apply_bevy_components(entity, w);

                    let handle_scene: Handle<Scene> = w.resource::<AssetServer>().load(&asset_path);
                    w.entity_mut(entity).insert((
                        SceneRoot(handle_scene),
                        PendingModelScene {
                            loaded_signal_id: spawn_copy.loaded_signal_id,
                            queue: q_for_comp,
                        },
                    ));
                });

                let ud = lua_ctx.create_userdata(model)?;
                lua_ctx
                    .named_registry_value::<mlua::Table>("__instance_cache")?
                    .set(handle, ud.clone())?;
                Ok(ud)
            })?,
        )?;

        lua.globals().set("Model", t)
    }
}

/// Scan the GLTF tree and parse into parent/children model
pub fn sync_model_hierarchy_system(
    mut commands: Commands,
    vm: NonSend<LuaVm>,
    query: Query<(Entity, &LuauHandle, &SceneInstance, &PendingModelScene)>,
    scene_spawner: Res<SceneSpawner>,
    name_query: Query<&Name>,
    mesh_query: Query<(), With<Mesh3d>>,
    parent_query: Query<&ChildOf>,
    mut handle_map: ResMut<HandleMap>,
) {
    let lua = &vm.lua;
    let Ok(cache) = lua.named_registry_value::<mlua::Table>("__instance_cache") else {
        return;
    };

    for (root_entity, root_handle, scene_instance, pending_data) in query.iter() {
        if !scene_spawner.instance_is_ready(**scene_instance) {
            continue;
        }

        commands.entity(root_entity).remove::<PendingModelScene>();

        let spawned_entities: Vec<Entity> = scene_spawner
            .iter_instance_entities(**scene_instance)
            .collect();
        let mut entity_to_handle = std::collections::HashMap::new();
        entity_to_handle.insert(root_entity, root_handle.0);

        for &entity in &spawned_entities {
            let handle = next_handle();
            entity_to_handle.insert(entity, handle);
            handle_map.insert(handle, entity, None);
            commands.entity(entity).insert(LuauHandle(handle));

            let name = name_query
                .get(entity)
                .map(|n| n.as_str())
                .unwrap_or("Object")
                .to_string();
            let is_mesh = mesh_query.contains(entity);

            if is_mesh {
                let touch_id = LuaSignal::new(lua).unwrap().id;
                let mut mesh_part = LuaMeshPart {
                    base_part_data: BasePartData::new(handle, pending_data.queue.clone(), touch_id),
                    mesh_id: "".to_string(),
                    collision_fidelity: 0,
                };
                mesh_part.base_part_data.base.name = name;
                let ud = lua.create_userdata(mesh_part).unwrap();
                cache.set(handle, ud).unwrap();
            } else {
                let mut model = LuaModel {
                    base: InstanceData::new(handle, pending_data.queue.clone(), "Model"),
                    collision_fidelity: 0,
                    loaded_signal_id: 0,
                };
                model.base.name = name;
                let ud = lua.create_userdata(model).unwrap();
                cache.set(handle, ud).unwrap();
            }
        }

        for &entity in &spawned_entities {
            if let Ok(parent) = parent_query.get(entity) {
                if let Some(&child_handle) = entity_to_handle.get(&entity) {
                    if let Some(&parent_handle) = entity_to_handle.get(&parent.get()) {
                        if let Ok(child_ud) = cache.get::<mlua::AnyUserData>(child_handle) {
                            if let Ok(parent_ud) = cache.get::<mlua::AnyUserData>(parent_handle) {
                                let _ = child_ud
                                    .call_method::<()>("__set_parent_silent", parent_handle);
                                let _ =
                                    parent_ud.call_method::<()>("__add_child_handle", child_handle);
                            }
                        }
                    }
                }
            }
        }

        let _ = LuaSignal {
            id: pending_data.loaded_signal_id,
        }
        .fire(lua, ());
    }
}
