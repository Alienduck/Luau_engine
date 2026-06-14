use bevy::{pbr::FogFalloff, post_process::bloom::Bloom, prelude::*};
use luau_classes::{
    instances::bloom_effect::LuauBloom,
    types::{
        color3::LuaColor3,
        instance::{CloneableInstance, InstanceData},
    },
};
use luau_runtime::{
    bridge::{
        handle::{HandleMap, next_handle},
        queue::EngineQueue,
    },
    registry::LuaModule,
};
use mlua::{Lua, UserData, UserDataFields};
use std::f32::consts::{FRAC_PI_2, TAU};

#[derive(Component)]
pub struct LightingRoot;

#[derive(Clone)]
pub struct LuaLighting {
    pub base: InstanceData,
    pub ambient: LuaColor3,
    pub brightness: f32,
    pub global_shadows: bool,
    pub clock_time: f32,
    pub fog_color: LuaColor3,
    pub fog_start: f32,
    pub fog_end: f32,
}

fn format_time(t: f32) -> String {
    let h = t.floor() as u32 % 24;
    let m = ((t * 60.0).floor() as u32) % 60;
    let s = ((t * 3600.0).floor() as u32) % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn parse_time(s: &str) -> f32 {
    let p: Vec<&str> = s.split(':').collect();
    let mut t = 0.0;
    if !p.is_empty() {
        t += p[0].parse::<f32>().unwrap_or(0.0);
    }
    if p.len() > 1 {
        t += p[1].parse::<f32>().unwrap_or(0.0) / 60.0;
    }
    if p.len() > 2 {
        t += p[2].parse::<f32>().unwrap_or(0.0) / 3600.0;
    }
    t % 24.0
}

fn update_sun_rotation(w: &mut World, time: f32) {
    if let Ok(mut transform) = w
        .query_filtered::<&mut Transform, With<DirectionalLight>>()
        .single_mut(w)
    {
        let angle = (time / 24.0) * TAU;
        transform.rotation = Quat::from_euler(EulerRot::XYZ, angle - FRAC_PI_2, 0.0, 0.0);
    }
}

fn update_fog(w: &mut World, c: LuaColor3, s: f32, e: f32) {
    if let Ok(cam) = w.query_filtered::<Entity, With<Camera3d>>().single_mut(w) {
        w.entity_mut(cam).insert(DistanceFog {
            color: Color::srgba(c.r, c.g, c.b, 1.0),
            falloff: FogFalloff::Linear { start: s, end: e },
            ..default()
        });
    }
}

pub fn sync_post_processing_system(
    lighting_query: Query<Entity, With<LightingRoot>>,
    effect_query: Query<(&LuauBloom, &ChildOf)>,
    mut camera_query: Query<&mut Bloom, With<Camera3d>>,
) {
    if lighting_query.is_empty() || camera_query.is_empty() {
        return;
    }

    let Ok(lighting_entity) = lighting_query.single() else {
        return;
    };
    let Ok(mut bloom_settings) = camera_query.single_mut() else {
        return;
    };

    let mut active_bloom = None;
    for (bloom, parent) in effect_query.iter() {
        if parent.0 == lighting_entity {
            active_bloom = Some(bloom);
            break;
        }
    }

    if let Some(bloom) = active_bloom {
        bloom_settings.intensity = (bloom.intensity * 0.15).clamp(0.0, 1.0);

        bloom_settings.low_frequency_boost = (bloom.size / 56.0).clamp(0.0, 1.0);

        bloom_settings.prefilter.threshold = bloom.threshold.max(1.0);
    } else {
        bloom_settings.intensity = 0.0;
    }
}

impl CloneableInstance for LuaLighting {
    fn base(&self) -> &InstanceData {
        &self.base
    }
    fn base_mut(&mut self) -> &mut InstanceData {
        &mut self.base
    }
    fn apply_bevy_components(&self, entity: Entity, w: &mut World) {
        if let Ok(mut e) = w.get_entity_mut(entity) {
            e.insert(LightingRoot);
        }
    }
}

impl UserData for LuaLighting {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.base.name.clone()));
        fields.add_field_method_set("Name", |_, this, v: String| {
            this.base.set_name(v);
            Ok(())
        });
        fields.add_field_method_get("Parent", |_, _| Ok(None::<mlua::AnyUserData>));
        fields.add_field_method_set("Parent", |_, _, _: Option<mlua::AnyUserData>| {
            Err(mlua::Error::runtime("Lighting cannot be parented"))
        });

        fields.add_field_method_get("Ambient", |_, this| Ok(this.ambient));
        fields.add_field_method_set("Ambient", |_, this, c: LuaColor3| {
            this.ambient = c;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Ok(mut ambient) = w.query::<&mut AmbientLight>().single_mut(w) {
                        ambient.color = Color::srgba(c.r, c.g, c.b, 1.0);
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("Brightness", |_, this| Ok(this.brightness));
        fields.add_field_method_set("Brightness", |_, this, b: f32| {
            this.brightness = b;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Ok(mut dir_light) = w.query::<&mut DirectionalLight>().single_mut(w) {
                        dir_light.illuminance = b * 1000.0;
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("GlobalShadows", |_, this| Ok(this.global_shadows));
        fields.add_field_method_set("GlobalShadows", |_, this, s: bool| {
            this.global_shadows = s;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    if let Ok(mut dir_light) = w.query::<&mut DirectionalLight>().single_mut(w) {
                        dir_light.shadows_enabled = s;
                    }
                }));
            Ok(())
        });

        fields.add_field_method_get("ClockTime", |_, this| Ok(this.clock_time));
        fields.add_field_method_set("ClockTime", |_, this, mut t: f32| {
            t %= 24.0;
            if t < 0.0 {
                t += 24.0;
            }
            this.clock_time = t;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    update_sun_rotation(w, t);
                }));
            Ok(())
        });

        fields.add_field_method_get("TimeOfDay", |_, this| Ok(format_time(this.clock_time)));
        fields.add_field_method_set("TimeOfDay", |_, this, s: String| {
            let t = parse_time(&s);
            this.clock_time = t;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    update_sun_rotation(w, t);
                }));
            Ok(())
        });

        fields.add_field_method_get("FogColor", |_, this| Ok(this.fog_color));
        fields.add_field_method_set("FogColor", |_, this, c: LuaColor3| {
            this.fog_color = c;
            let s = this.fog_start;
            let e = this.fog_end;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    update_fog(w, c, s, e);
                }));
            Ok(())
        });

        fields.add_field_method_get("FogStart", |_, this| Ok(this.fog_start));
        fields.add_field_method_set("FogStart", |_, this, s: f32| {
            this.fog_start = s;
            let c = this.fog_color;
            let e = this.fog_end;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    update_fog(w, c, s, e);
                }));
            Ok(())
        });

        fields.add_field_method_get("FogEnd", |_, this| Ok(this.fog_end));
        fields.add_field_method_set("FogEnd", |_, this, e: f32| {
            this.fog_end = e;
            let c = this.fog_color;
            let s = this.fog_start;
            this.base
                .queue
                .0
                .lock()
                .unwrap()
                .push(Box::new(move |w: &mut World| {
                    update_fog(w, c, s, e);
                }));
            Ok(())
        });
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        luau_classes::impl_instance_userdata!(methods);
        methods.add_method("Clone", |_, _, ()| -> mlua::Result<()> {
            Err(mlua::Error::runtime("Lighting cannot be cloned"))
        });
    }
}

pub struct LightingModule;

impl LuaModule for LightingModule {
    fn name() -> &'static str {
        "Lighting"
    }
    fn register(lua: &Lua, queue: &EngineQueue) -> mlua::Result<()> {
        let handle = next_handle();
        let q = queue.clone();

        q.0.lock().unwrap().push(Box::new(move |w: &mut World| {
            let entity = w
                .spawn((LightingRoot, Transform::default(), Visibility::Inherited))
                .id();
            w.resource_mut::<HandleMap>().insert(handle, entity, None);
        }));

        let lighting = LuaLighting {
            base: InstanceData::new(handle, queue.clone(), "Lighting"),
            ambient: LuaColor3 {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            brightness: 10.0,
            global_shadows: true,
            clock_time: 14.0,
            fog_color: LuaColor3 {
                r: 0.75,
                g: 0.75,
                b: 0.75,
            },
            fog_start: 0.0,
            fog_end: 100_000.0,
        };

        let userdata = lua.create_userdata(lighting)?;
        lua.set_named_registry_value("__lighting_instance", userdata.clone())?;
        lua.globals().set("Lighting", userdata)?;
        Ok(())
    }
}
