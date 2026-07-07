use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use luau_runtime::{
    bridge::{handle::LuauHandle, queue::EngineQueue},
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
        fields.add_field_method_set("FilterDescendantInstances", |_, this, v: Vec<u64>| {
            this.filter_descendant_instances = v;
            Ok(())
        });
        fields.add_field_method_get("FilterType", |_, this| Ok(this.filter_type as u8));
        fields.add_field_method_set("FilterType", |_, this, v: u8| {
            this.filter_type = RaycastFilterType::from(v);
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
    let max_toi = direction.length();
    let dir = direction.normalize_or_zero();
    if dir == Vec3::ZERO {
        return Ok(None);
    }
    let mut query = world.query::<RapierContext>();
    let rapier_context_item = query
        .single(world)
        .map_err(|_| mlua::Error::runtime("RapierContext missing"))?;
    let rapier_context = RapierContext {
        simulation: rapier_context_item.simulation,
        colliders: rapier_context_item.colliders,
        joints: rapier_context_item.joints,
        rigidbody_set: rapier_context_item.rigidbody_set,
    };
    let mut filter = QueryFilter::new();

    let predicate;

    if let Some(p) = &params {
        predicate = move |entity: Entity| match p.filter_type {
            RaycastFilterType::Exclude => {
                !p.filter_descendant_instances.contains(&entity.to_bits())
            }
            RaycastFilterType::Include => p.filter_descendant_instances.contains(&entity.to_bits()),
        };
        filter = filter.predicate(&predicate);
    }
    Ok(rapier_context
        .cast_ray_and_get_normal(origin, dir, max_toi, true, filter)
        .map(|(entity, intersection)| {
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
                position: intersection.point,
                distance: intersection.time_of_impact,
                material,
                normal: intersection.normal,
            }
        }))
}
