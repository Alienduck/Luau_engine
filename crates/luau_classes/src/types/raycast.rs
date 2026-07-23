use bevy::{platform::collections::HashSet, prelude::*};
use engine_core::definitions::services::PhysicsCollisionGroups;
use luau_runtime::{
    bridge::{
        handle::{HandleMap, LuauHandle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData};

use crate::types::{
    enums::{BasePartMaterial, RaycastFilterType},
    vector3::LuaVector3,
};

#[derive(Clone)]
pub struct RaycastParams {
    pub filter_descendant_instances: Vec<u64>,
    pub filter_type: RaycastFilterType,
    pub restpect_collider: bool,
    pub collision_group: String,
}

impl mlua::FromLua for RaycastParams {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            other => Err(mlua::Error::runtime(format!(
                "expected RaycastParams, got {}",
                other.type_name()
            ))),
        }
    }
}

impl UserData for RaycastParams {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("FilterDescendantInstances", |_, this| {
            Ok(this.filter_descendant_instances.clone())
        });
        fields.add_field_method_set("FilterDescendantInstances", |_, this, v: mlua::Table| {
            let mut handles = Vec::new();
            for pair in v.pairs::<mlua::Integer, mlua::AnyUserData>() {
                if let Ok((_, ud)) = pair {
                    if let Ok(part) = ud.borrow::<crate::instances::part::LuaPart>() {
                        handles.push(part.data.base.handle);
                    } else if let Ok(mesh) = ud.borrow::<crate::instances::mesh_part::LuaMeshPart>()
                    {
                        handles.push(mesh.base_part_data.base.handle);
                    }
                }
            }
            this.filter_descendant_instances = handles;
            Ok(())
        });
        fields.add_field_method_get("FilterType", |_, this| Ok(this.filter_type as u8));
        fields.add_field_method_set("FilterType", |_, this, v: u8| {
            this.filter_type = RaycastFilterType::from(v);
            Ok(())
        });
        fields.add_field_method_get("RespectCollider", |_, this| Ok(this.restpect_collider));
        fields.add_field_method_set("RespectCollider", |_, this, v: bool| {
            this.restpect_collider = v;
            Ok(())
        });
    }
}

#[derive(Clone)]
pub struct RaycastResult {
    pub instance: u64,
    pub position: Vec3,
    pub distance: f32,
    pub material: BasePartMaterial,
    pub normal: Vec3,
}

impl UserData for RaycastResult {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Instance", |lua, this| {
            let cache = lua.named_registry_value::<mlua::Table>("__instance_cache")?;
            cache.get::<mlua::AnyUserData>(this.instance)
        });
        fields.add_field_method_get("Position", |_, this| -> mlua::Result<LuaVector3> {
            Ok(this.position.into())
        });
        fields.add_field_method_get("Distance", |_, this| Ok(this.distance));
        fields.add_field_method_get("Material", |_, this| Ok(this.material as u8));
        fields.add_field_method_get("Normal", |_, this| {
            Ok(LuaVector3 {
                x: this.normal.x,
                y: this.normal.y,
                z: this.normal.z,
            })
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(
            mlua::MetaMethod::ToString,
            |lua, this, ()| -> mlua::Result<String> {
                let cache = lua.named_registry_value::<mlua::Table>("__instance_cache")?;

                let instance_name = if let Ok(ud) = cache.get::<mlua::AnyUserData>(this.instance) {
                    crate::types::instance::instance_name_from_any(&ud)
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "None".to_string()
                };

                Ok(format!(
                    "RaycastResult {{\n    Instance: {},\n    Position: {},\n    Distance: {},\n    Material: {},\n    Normal: {}\n}}",
                    instance_name,
                    this.position,
                    this.distance,
                    this.material.as_ref(),
                    this.normal
                ))
            },
        );
    }
}

pub struct RaycastModule;

impl LuaModule for RaycastModule {
    fn name() -> &'static str {
        "RaycastParams"
    }

    fn register(lua: &Lua, _queue: &EngineQueue) -> mlua::Result<()> {
        let params_table = lua.create_table()?;

        let new_fn = lua.create_function(|_, ()| {
            Ok(RaycastParams {
                filter_descendant_instances: Vec::new(),
                filter_type: RaycastFilterType::Exclude,
                restpect_collider: true,
                collision_group: "Default".into(),
            })
        })?;

        params_table.set("new", new_fn)?;
        lua.globals().set("RaycastParams", params_table)?;

        Ok(())
    }
}

pub fn workspace_raycast(
    lua: &mlua::Lua,
    world: &mut World,
    origin: Vec3,
    direction: Vec3,
    params: Option<RaycastParams>,
) -> mlua::Result<Option<RaycastResult>> {
    let Ok(dir) = Dir3::new(direction) else {
        return Err(mlua::Error::runtime(format!(
            "Failed to parse Vec3 into Dir3, the giving Vector was probably on a null direction"
        )));
    };
    let max_toi = direction.length();
    let mut entity_filter_list = HashSet::new();
    let mut filter_type = RaycastFilterType::Exclude;
    let mut respect_collider = true;
    let mut ray_group_id = 0;
    let has_params = params.is_some();
    if let Some(p) = &params {
        filter_type = p.filter_type.clone();
        respect_collider = p.restpect_collider;
        if let Some(registry) = world.get_resource::<PhysicsCollisionGroups>() {
            if let Some(&id) = registry.groups.get(&p.collision_group) {
                ray_group_id = id;
            }
        }
        let handle_map = world.resource::<HandleMap>();
        let mut stack = Vec::new();
        for handle in &p.filter_descendant_instances {
            if let Some(entity) = handle_map.get_entity(*handle) {
                stack.push(entity);
            }
        }
        while let Some(current_entity) = stack.pop() {
            entity_filter_list.insert(current_entity);
            if let Some(children) = world.get::<Children>(current_entity) {
                stack.extend(children.iter());
            }
        }
    }
    let mut query_filter = avian3d::prelude::SpatialQueryFilter::default();
    if let Some(registry) = world.get_resource::<PhysicsCollisionGroups>() {
        query_filter = query_filter.with_mask(registry.masks[ray_group_id as usize]);
    }
    let mut state = bevy::ecs::system::SystemState::<(
        avian3d::prelude::SpatialQuery,
        Query<(), With<avian3d::prelude::Sensor>>,
    )>::new(world);
    let Ok((spatial_query, sensors)) = state.get(world) else {
        return Err(mlua::Error::runtime(format!(
            "Failed to get spatial query state"
        )));
    };
    let predicate = |entity: Entity| -> bool {
        if respect_collider && sensors.contains(entity) {
            return false;
        }
        if has_params {
            match filter_type {
                RaycastFilterType::Exclude => !entity_filter_list.contains(&entity),
                RaycastFilterType::Include => entity_filter_list.contains(&entity),
            }
        } else {
            true
        }
    };
    let hit =
        spatial_query.cast_ray_predicate(origin, dir, max_toi, true, &query_filter, &predicate);
    Ok(hit.map(|intersection| {
        let entity = intersection.entity;
        let handle = world.get::<LuauHandle>(entity).map_or(0, |h| h.0);
        let mut material = BasePartMaterial::Plastic;
        if handle != 0 {
            if let Ok(cache) = lua.named_registry_value::<mlua::Table>("__instance_cache") {
                if let Ok(ud) = cache.get::<mlua::AnyUserData>(handle) {
                    if let Ok(part) = ud.borrow::<crate::instances::part::LuaPart>() {
                        material = part.data.material.clone();
                    } else if let Ok(mesh_part) =
                        ud.borrow::<crate::instances::mesh_part::LuaMeshPart>()
                    {
                        material = mesh_part.base_part_data.material.clone();
                    }
                }
            }
        }
        RaycastResult {
            instance: handle,
            position: origin + dir * intersection.distance,
            distance: intersection.distance,
            material,
            normal: intersection.normal,
        }
    }))
}
