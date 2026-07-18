use avian3d::prelude::*;
use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use engine_core::{
    components::{LuauAtmosphere, LuauBloom, LuauCharacterController},
    resource::PhysicsCollisionGroups,
};

/// A single mutation enqueued by a Luau setter.
///
/// Sizing note: the largest variant (SetMaterial) is ~40 bytes on 64-bit;
/// the Vec stores them contiguously, keeping cache lines full.
pub enum EngineCommand {
    /// `Transform::translation`
    SetTranslation {
        handle: u64,
        translation: Vec3,
    },
    /// `Transform::scale`
    SetScale {
        handle: u64,
        scale: Vec3,
    },
    /// `Transform::rotation`
    SetRotation {
        handle: u64,
        rotation: Quat,
    },
    /// Full CFrame (translation + rotation in one shot)
    SetCFrame {
        handle: u64,
        translation: Vec3,
        rotation: Quat,
    },
    /// `StandardMaterial::base_color` + `alpha_mode`
    SetBaseColor {
        handle: u64,
        r: f32,
        g: f32,
        b: f32,
        /// 0.0 = opaque
        alpha: f32,
    },
    SetShadowCast {
        handle: u64,
        cast: bool,
    },
    SetShadowReceiver {
        handle: u64,
        receive: bool,
    },
    /// `StandardMaterial::emissive`
    SetEmissive {
        handle: u64,
        r: f32,
        g: f32,
        b: f32,
    },
    /// `Visibility`
    SetVisibility {
        handle: u64,
        visible: bool,
    },
    /// `bevy_ui::Node` layout (position + size computed upstream)
    SetNodeLayout {
        handle: u64,
        left: Val,
        top: Val,
        width: Val,
        height: Val,
        margin_left: Val,
        margin_top: Val,
    },
    SetUiRotation {
        handle: u64,
        rotation: f32,
    },
    SetZindex {
        handle: u64,
        zindex: i32,
    },
    /// `BackgroundColor`
    SetBackgroundColor {
        handle: u64,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    /// `Text::0`
    SetText {
        handle: u64,
        text: String,
    },
    /// `TextColor`
    SetTextColor {
        handle: u64,
        r: f32,
        g: f32,
        b: f32,
    },
    /// `TextFont::font_size`
    SetFontSize {
        handle: u64,
        size: f32,
    },
    /// `RigidBody` insert/remove on parent entity
    SetRigidBody {
        handle: u64,
        dynamic: bool,
        gravity_scale: f32,
    },
    /// `RigidBody` set the gravity scale
    SetRigidBodyGravityScale {
        handle: u64,
        gravity_scale: f32,
    },
    SetRigidbodyMass {
        handle: u64,
        mass: f32,
    },
    ApplyRigidbodyImpulse {
        handle: u64,
        impulse: Vec3,
    },
    SetRigidbodyAngularVelocity {
        handle: u64,
        angular_velocity: Vec3,
    },
    /// `Collider` set if the collider is active or not
    SetColliderCollide {
        handle: u64,
        can_collide: bool,
    },
    /// `Collider` set the friction of the collider
    SetColliderFriction {
        handle: u64,
        friction: f32,
    },
    /// `Collider` set the restitution of the collider
    SetColliderRestitution {
        handle: u64,
        restitution: f32,
    },
    SetCollisionGroup {
        handle: u64,
        group: String,
    },
    SetCollisionGroups {
        handle: u64,
        memberships: u32,
        filters: u32,
    },
    /// `ImageNode` insert an ImageNode
    SetImageNode {
        handle: u64,
        asset_path: String,
    },
    /// `AmbientLight` set the ambient light color
    SetLightingColor {
        color: Color,
    },
    /// `DirectionalLight` set the directional light luminance
    SetLightingBrightness {
        illuminance: f32,
    },
    /// `DirectionalLight` set the directional light global shadows
    SetLightingGlobalShadows {
        enabled: bool,
    },
    RegisterCollisionGroup {
        name: String,
    },
    SetCollisionGroupCollidable {
        group1: String,
        group2: String,
        collidable: bool,
    },
    /// `Bloom` set the bloom intensity
    SetBloomIntensity {
        handle: u64,
        intensity: f32,
    },
    /// `Bloom` set the bloom size
    SetBloomSize {
        handle: u64,
        size: f32,
    },
    /// `Bloom` set the bloom size
    SetBloomThreshold {
        handle: u64,
        threshold: f32,
    },
    /// `Atmosphere` set the atmosphere density
    SetAtmosphereDensity {
        handle: u64,
        density: f32,
    },
    /// `Atmosphere` set the atmosphere Color
    SetAtmosphereColor {
        handle: u64,
        color: Color,
    },
    /// `Atmosphere` set the atmosphere decay
    SetAtmosphereDecay {
        handle: u64,
        decay: Color,
    },
    /// `Atmosphere` set the atmosphere glare
    SetAtmosphereGlare {
        handle: u64,
        glare: f32,
    },
    /// `Atmosphere` set the atmosphere haze
    SetAtmosphereHaze {
        handle: u64,
        haze: f32,
    },
    SetCharacterMovement {
        handle: u64,
        movement: Vec3,
    },
    SetCharacterJump {
        handle: u64,
        jump: bool,
    },
    SetCharacterJumpPower {
        handle: u64,
        jump_power: f32,
    },
    /// `KinematicCharacterController` set the character clip mode
    SetCharacterNoClip {
        handle: u64,
        no_clip: bool,
    },
    /// Despawn entity + remove from HandleMap
    Despawn {
        handle: u64,
    },
    /// Re-parent in the Bevy hierarchy
    SetParent {
        child_handle: u64,
        parent_handle: Option<u64>,
    },
    /// Using the `AssetServer` to load an asset from a given path
    LoadAsset {
        handle: u64,
        asset_path: String,
    },
    EnableRenderCollisionDebug {
        enable: bool,
    },
    SetModeRenderCollisionDebug {
        mode: u8,
    },
    /// Any mutation that cannot be expressed as a variant above.
    /// This is the escape hatch — keep usage minimal.
    Raw(Box<dyn FnOnce(&mut World) + Send + Sync>),
}

pub type CommandBuffer = Vec<EngineCommand>;

/// Cheaply-cloneable handle to the shared command buffer.
///
/// Replace every `Arc<Mutex<Vec<WorldCommand>>>` in the codebase with this.
#[derive(Clone)]
pub struct EngineQueue(pub Sender<EngineCommand>);

impl EngineQueue {
    /// Push a typed command.  Costs one `borrow_mut()` (single flag check,
    /// never contended) and one `Vec::push` (amortised O(1)).
    #[inline]
    pub fn push(&self, cmd: EngineCommand) {
        let _ = self.0.send(cmd);
    }

    /// Push an untyped closure.  Allocates one `Box`; use only for rare,
    /// complex mutations that have no matching variant.
    #[inline]
    pub fn push_raw<F>(&self, f: F)
    where
        F: FnOnce(&mut World) + 'static + Send + Sync,
    {
        self.0.send(EngineCommand::Raw(Box::new(f))).ok();
    }
}

/// Bevy resource that holds the shared command buffer.
///
/// Must only be accessed from the main thread.  The `Send`/`Sync` impls
/// are handled by the crate `crossbeam_channel`
#[derive(Resource)]
pub struct EngineQueueResource(pub Receiver<EngineCommand>);

unsafe impl Send for EngineQueueResource {}
unsafe impl Sync for EngineQueueResource {}

/// Exclusive Bevy system — drains and applies every queued command.
///
/// Runs once per frame, after Luau scripts have been ticked.
/// No lock, no allocation: just a Vec drain and a match dispatch.
pub fn process_engine_queue(world: &mut World) {
    let receiver = world.resource::<EngineQueueResource>().0.clone();
    for cmd in receiver.try_iter() {
        apply_command(world, cmd);
    }
}

#[inline]
fn apply_command(world: &mut World, cmd: EngineCommand) {
    use crate::bridge::handle::HandleMap;

    match cmd {
        EngineCommand::SetTranslation {
            handle,
            translation,
        } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut t) = world.get_mut::<Transform>(e) {
                    t.translation = translation;
                }
            }
        }
        EngineCommand::SetScale { handle, scale } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut t) = world.get_mut::<Transform>(e) {
                    t.scale = scale;
                }
            }
        }
        EngineCommand::SetRotation { handle, rotation } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut t) = world.get_mut::<Transform>(e) {
                    t.rotation = rotation;
                }
            }
        }
        EngineCommand::SetCFrame {
            handle,
            translation,
            rotation,
        } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut t) = world.get_mut::<Transform>(e) {
                    t.translation = translation;
                    t.rotation = rotation;
                }
            }
        }
        EngineCommand::SetBaseColor {
            handle,
            r,
            g,
            b,
            alpha,
        } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mat_h) = world
                    .get::<MeshMaterial3d<StandardMaterial>>(e)
                    .map(|m| m.0.clone())
                {
                    if let Some(mut mat) = world
                        .resource_mut::<Assets<StandardMaterial>>()
                        .get_mut(&mat_h)
                    {
                        mat.base_color = Color::srgba(r, g, b, alpha);
                        mat.alpha_mode = if alpha < 1.0 {
                            AlphaMode::Blend
                        } else {
                            AlphaMode::Opaque
                        };
                    }
                }
            }
        }
        EngineCommand::SetShadowCast { handle, cast } => {
            use bevy::light::NotShadowCaster;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    if cast {
                        if em.contains::<NotShadowCaster>() {
                            em.remove::<NotShadowCaster>();
                        }
                    } else {
                        if !em.contains::<NotShadowCaster>() {
                            em.insert(NotShadowCaster);
                        }
                    }
                }
            }
        }
        EngineCommand::SetShadowReceiver { handle, receive } => {
            use bevy::light::NotShadowReceiver;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    if receive {
                        if em.contains::<NotShadowReceiver>() {
                            em.remove::<NotShadowReceiver>();
                        }
                    } else {
                        if !em.contains::<NotShadowReceiver>() {
                            em.insert(NotShadowReceiver);
                        }
                    }
                }
            }
        }
        EngineCommand::SetEmissive { handle, r, g, b } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mat_h) = world
                    .get::<MeshMaterial3d<StandardMaterial>>(e)
                    .map(|m| m.0.clone())
                {
                    if let Some(mut mat) = world
                        .resource_mut::<Assets<StandardMaterial>>()
                        .get_mut(&mat_h)
                    {
                        mat.emissive = LinearRgba::rgb(r, g, b);
                    }
                }
            }
        }
        EngineCommand::SetVisibility { handle, visible } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut v) = world.get_mut::<Visibility>(e) {
                    *v = if visible {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
        EngineCommand::SetNodeLayout {
            handle,
            left,
            top,
            width,
            height,
            margin_left,
            margin_top,
        } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut n) = world.get_mut::<Node>(e) {
                    n.left = left;
                    n.top = top;
                    n.width = width;
                    n.height = height;
                    n.margin.left = margin_left;
                    n.margin.top = margin_top;
                }
            }
        }
        EngineCommand::SetUiRotation { handle, rotation } => {
            use bevy::ui::UiTransform;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut ut) = world.get_mut::<UiTransform>(e) {
                    ut.rotation = Rot2::degrees(rotation);
                } else {
                    if let Ok(mut em) = world.get_entity_mut(e) {
                        em.insert(UiTransform::from_rotation(Rot2::degrees(rotation)));
                    }
                }
            }
        }
        EngineCommand::SetZindex { handle, zindex } => {
            use bevy::ui::ZIndex;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut zi) = world.get_mut::<ZIndex>(e) {
                    zi.0 = zindex;
                } else {
                    if let Ok(mut em) = world.get_entity_mut(e) {
                        em.insert(ZIndex(zindex));
                    }
                }
            }
        }
        EngineCommand::SetBackgroundColor { handle, r, g, b, a } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut bg) = world.get_mut::<BackgroundColor>(e) {
                    bg.0 = Color::srgba(r, g, b, a);
                }
            }
        }
        EngineCommand::SetText { handle, text } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut t) = world.get_mut::<Text>(e) {
                    t.0 = text;
                }
            }
        }
        EngineCommand::SetTextColor { handle, r, g, b } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut tc) = world.get_mut::<TextColor>(e) {
                    tc.0 = Color::srgba(r, g, b, 1.0);
                }
            }
        }
        EngineCommand::SetFontSize { handle, size } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut tf) = world.get_mut::<TextFont>(e) {
                    tf.font_size = FontSize::Px(size);
                }
            }
        }
        EngineCommand::SetRigidBody {
            handle,
            dynamic,
            gravity_scale,
        } => {
            use avian3d::prelude::{GravityScale, LinearVelocity, RigidBody, Sleeping};
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    em.insert(if dynamic {
                        RigidBody::Dynamic
                    } else {
                        RigidBody::Static
                    })
                    .insert(GravityScale(gravity_scale))
                    .insert(LinearVelocity::default());

                    if let Some(_) = em.get::<Sleeping>() {
                        em.remove::<Sleeping>();
                    }
                }
            }
        }
        EngineCommand::SetRigidBodyGravityScale {
            handle,
            gravity_scale,
        } => {
            use avian3d::prelude::{GravityScale, Sleeping};
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    em.insert(GravityScale(gravity_scale));
                    if let Some(_) = em.get::<Sleeping>() {
                        em.remove::<Sleeping>();
                    }
                }
            }
        }
        EngineCommand::SetRigidbodyMass { handle, mass } => {
            use avian3d::prelude::Mass;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    em.insert(Mass(mass));
                }
            }
        }
        EngineCommand::ApplyRigidbodyImpulse { handle, impulse } => {
            use avian3d::prelude::{LinearVelocity, Mass};
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    let mass = em.get::<Mass>().map_or(1.0, |m| m.0);
                    if let Some(mut lv) = em.get_mut::<LinearVelocity>() {
                        lv.0 += impulse / mass;
                    } else {
                        em.insert(LinearVelocity(impulse / mass));
                    }
                }
            }
        }
        EngineCommand::SetRigidbodyAngularVelocity {
            handle,
            angular_velocity,
        } => {
            use avian3d::prelude::AngularVelocity;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    if let Some(mut v) = em.get_mut::<AngularVelocity>() {
                        v.0 = angular_velocity;
                    } else {
                        em.insert(AngularVelocity(angular_velocity));
                    }
                }
            }
        }
        EngineCommand::SetColliderCollide {
            handle,
            can_collide,
        } => {
            use avian3d::prelude::Sensor;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                let mut entity_mut = world.entity_mut(e);
                if can_collide {
                    entity_mut.remove::<Sensor>();
                } else {
                    entity_mut.insert(Sensor::default());
                }
            }
        }
        EngineCommand::SetColliderFriction { handle, friction } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut f) = world.get_mut::<Friction>(e) {
                    *f = Friction::new(friction);
                } else {
                    world.entity_mut(e).insert(Friction::new(friction));
                }
            }
        }
        EngineCommand::SetColliderRestitution {
            handle,
            restitution,
        } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if world.get::<Restitution>(e).is_some() {
                    world.get_mut::<Restitution>(e).unwrap().coefficient = restitution;
                } else {
                    world.entity_mut(e).insert(Restitution::new(restitution));
                }
            }
        }
        EngineCommand::SetCollisionGroup { handle, group } => {
            let mut memberships = 1;
            let mut filters = u32::MAX;
            if let Some(registry) = world.get_resource::<PhysicsCollisionGroups>() {
                let id = registry.groups.get(&group).copied().unwrap_or(0);
                memberships = 1 << id;
                filters = registry.masks[id as usize];
            }
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    em.insert(avian3d::prelude::CollisionLayers::from_bits(
                        memberships,
                        filters,
                    ));
                }
            }
        }
        EngineCommand::SetCollisionGroups {
            handle,
            memberships,
            filters,
        } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    em.insert(CollisionLayers::from_bits(memberships, filters));
                }
            }
        }
        EngineCommand::SetImageNode { handle, asset_path } => {
            let image_handle = world.resource::<AssetServer>().load(&asset_path);
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if world.get::<ImageNode>(e).is_some() {
                    world.get_mut::<ImageNode>(e).unwrap().image = image_handle;
                } else {
                    world.entity_mut(e).insert(ImageNode::new(image_handle));
                }
            }
        }
        EngineCommand::SetLightingColor { color } => {
            if let Ok(mut ambient) = world.query::<&mut AmbientLight>().single_mut(world) {
                ambient.color = color;
            }
        }
        EngineCommand::SetLightingBrightness { illuminance } => {
            if let Ok(mut d) = world.query::<&mut DirectionalLight>().single_mut(world) {
                d.illuminance = illuminance;
            }
        }
        EngineCommand::SetLightingGlobalShadows { enabled } => {
            if let Ok(mut d) = world.query::<&mut DirectionalLight>().single_mut(world) {
                d.shadow_maps_enabled = enabled;
            }
        }
        EngineCommand::RegisterCollisionGroup { name } => {
            if let Some(mut registry) = world.get_resource_mut::<PhysicsCollisionGroups>() {
                let next_id = registry.next_id;
                if !registry.groups.contains_key(&name) && registry.next_id < 32 {
                    registry.groups.insert(name, next_id);
                    registry.next_id += 1;
                }
            }
        }
        EngineCommand::SetCollisionGroupCollidable {
            group1,
            group2,
            collidable,
        } => {
            let mut id1_opt = None;
            let mut id2_opt = None;
            if let Some(registry) = world.get_resource::<PhysicsCollisionGroups>() {
                id1_opt = registry.groups.get(&group1).copied();
                id2_opt = registry.groups.get(&group2).copied();
            }

            if let (Some(id1), Some(id2)) = (id1_opt, id2_opt) {
                let masks = {
                    let mut registry = world.resource_mut::<PhysicsCollisionGroups>();
                    if collidable {
                        registry.masks[id1 as usize] |= 1 << id2;
                        registry.masks[id2 as usize] |= 1 << id1;
                    } else {
                        registry.masks[id1 as usize] &= !(1 << id2);
                        registry.masks[id2 as usize] &= !(1 << id1);
                    }
                    registry.masks.clone()
                };

                let mut updates = Vec::new();
                let mut query = world.query::<(Entity, &avian3d::prelude::CollisionLayers)>();

                for (entity, cg) in query.iter(world) {
                    let memberships = cg.memberships.to_be();
                    if memberships != 0 {
                        let group_id = memberships.trailing_zeros();
                        if group_id < 32 {
                            let new_cg = avian3d::prelude::CollisionLayers::from_bits(
                                memberships,
                                masks[group_id as usize],
                            );
                            updates.push((entity, new_cg));
                        }
                    }
                }

                for (entity, new_cg) in updates {
                    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                        entity_mut.insert(new_cg);
                    }
                }
            }
        }
        EngineCommand::SetBloomIntensity { handle, intensity } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut b) = world.get_mut::<LuauBloom>(e) {
                    b.intensity = intensity;
                }
            }
        }
        EngineCommand::SetBloomSize { handle, size } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut b) = world.get_mut::<LuauBloom>(e) {
                    b.size = size;
                }
            }
        }
        EngineCommand::SetBloomThreshold { handle, threshold } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut b) = world.get_mut::<LuauBloom>(e) {
                    b.threshold = threshold;
                }
            }
        }
        EngineCommand::SetAtmosphereDensity { handle, density } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut a) = world.get_mut::<LuauAtmosphere>(e) {
                    a.density = density;
                }
            }
        }
        EngineCommand::SetAtmosphereColor { handle, color } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut a) = world.get_mut::<LuauAtmosphere>(e) {
                    a.color = color;
                }
            }
        }
        EngineCommand::SetAtmosphereDecay { handle, decay } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut a) = world.get_mut::<LuauAtmosphere>(e) {
                    a.decay = decay;
                }
            }
        }
        EngineCommand::SetAtmosphereGlare { handle, glare } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut a) = world.get_mut::<LuauAtmosphere>(e) {
                    a.glare = glare;
                }
            }
        }
        EngineCommand::SetAtmosphereHaze { handle, haze } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut a) = world.get_mut::<LuauAtmosphere>(e) {
                    a.haze = haze;
                }
            }
        }
        EngineCommand::SetCharacterMovement { handle, movement } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut cc) = world.get_mut::<LuauCharacterController>(e) {
                    cc.velocity = movement;
                }
            }
        }
        EngineCommand::SetCharacterJump { handle, jump } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut cc) = world.get_mut::<LuauCharacterController>(e) {
                    cc.wants_to_jump = jump;
                }
            }
        }
        EngineCommand::SetCharacterJumpPower { handle, jump_power } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut cc) = world.get_mut::<LuauCharacterController>(e) {
                    cc.jump_power = jump_power;
                }
            }
        }
        EngineCommand::SetCharacterNoClip { handle, no_clip } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mut cc) = world.get_mut::<LuauCharacterController>(e) {
                    cc.no_clip = no_clip;
                }
            }
        }
        EngineCommand::Despawn { handle } => {
            if let Some(entry) = world.resource_mut::<HandleMap>().remove(handle) {
                if let Ok(e_mut) = world.get_entity_mut(entry.entity) {
                    e_mut.despawn();
                }
            }
        }
        EngineCommand::SetParent {
            child_handle,
            parent_handle,
        } => {
            let map = world.resource::<HandleMap>();
            let child_e = map.get_entity(child_handle);
            let parent_e = parent_handle.and_then(|h| map.get_entity(h));

            if let Some(child) = child_e {
                if let Ok(mut e_mut) = world.get_entity_mut(child) {
                    e_mut.remove::<ChildOf>();
                    e_mut.insert(if parent_e.is_some() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    });
                }
                if let Some(parent) = parent_e {
                    if let Ok(mut p_mut) = world.get_entity_mut(parent) {
                        p_mut.add_child(child);
                    }
                }
            }
        }
        EngineCommand::EnableRenderCollisionDebug { enable } => {
            if let Some(mut store) = world.get_resource_mut::<bevy::prelude::GizmoConfigStore>() {
                let (config, _) = store.config_mut::<avian3d::prelude::PhysicsGizmos>();
                config.enabled = enable;
            }
        }
        EngineCommand::SetModeRenderCollisionDebug { mode } => {
            use avian3d::prelude::{Collider, DebugRender, RigidBody};
            use bevy::math::Vec3;
            use bevy::prelude::{Color, Entity, Or, With};
            let mut updates = Vec::new();
            let mut query = world.query_filtered::<
                (Entity, Option<&DebugRender>),
                Or<(With<Collider>, With<RigidBody>)>
            >();
            for (entity, dr) in query.iter(world) {
                let mut new_dr = dr.cloned().unwrap_or_default();
                new_dr.collider_color = None;
                new_dr.aabb_color = None;
                new_dr.axis_lengths = None;
                match mode {
                    0 => new_dr.collider_color = Some(Color::srgb(0.0, 1.0, 0.0)),
                    1 => new_dr.axis_lengths = Some(Vec3::splat(1.0)),
                    2..=6 => {}
                    _ => new_dr.aabb_color = Some(Color::srgb(0.0, 0.0, 1.0)),
                }
                updates.push((entity, new_dr));
            }
            for (entity, new_dr) in updates {
                if let Ok(mut em) = world.get_entity_mut(entity) {
                    em.insert(new_dr);
                }
            }
        }
        EngineCommand::LoadAsset { handle, asset_path } => {
            let handle_scene: Handle<WorldAsset> =
                world.resource::<AssetServer>().load(&asset_path);
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                world.entity_mut(e).insert(WorldAssetRoot(handle_scene));
            }
        }
        EngineCommand::Raw(f) => f(world),
    }
}
