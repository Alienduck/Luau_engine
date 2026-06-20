use bevy::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A single mutation enqueued by a Luau setter.
///
/// Sizing note: the largest variant (SetMaterial) is ~40 bytes on 64-bit;
/// the Vec stores them contiguously, keeping cache lines full.
pub enum EngineCommand {
    /// `Transform::translation`
    SetTranslation { handle: u64, translation: Vec3 },
    /// `Transform::scale`
    SetScale { handle: u64, scale: Vec3 },
    /// `Transform::rotation`
    SetRotation { handle: u64, rotation: Quat },
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
    /// `StandardMaterial::emissive`
    SetEmissive { handle: u64, r: f32, g: f32, b: f32 },
    /// `Visibility`
    SetVisibility { handle: u64, visible: bool },
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
    /// `BackgroundColor`
    SetBackgroundColor {
        handle: u64,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    /// `Text::0`
    SetText { handle: u64, text: String },
    /// `TextColor`
    SetTextColor { handle: u64, r: f32, g: f32, b: f32 },
    /// `TextFont::font_size`
    SetFontSize { handle: u64, size: f32 },
    /// `RigidBody` insert/remove on parent entity
    SetRigidBody { handle: u64, dynamic: bool },
    /// `ImageNode` insert an ImageNode
    SetImageNode { handle: u64, asset_path: String },
    /// `AmbientLight` set the ambient light color
    SetLightingColor { color: Color },
    /// `DirectionalLight` set the directional light luminance
    SetLightingBrightness { illuminance: f32 },
    /// `DirectionalLight` set the directional light global shadows
    SetLightingGlobalShadows { enabled: bool },
    /// Despawn entity + remove from HandleMap
    Despawn { handle: u64 },
    /// Re-parent in the Bevy hierarchy
    SetParent {
        child_handle: u64,
        parent_handle: Option<u64>,
    },
    /// Using the `AssetServer` to load an asset from a given path
    LoadAsset { handle: u64, asset_path: String },
    /// Any mutation that cannot be expressed as a variant above.
    /// This is the escape hatch — keep usage minimal.
    Raw(Box<dyn FnOnce(&mut World)>),
}

pub type CommandBuffer = Vec<EngineCommand>;

/// Cheaply-cloneable handle to the shared command buffer.
///
/// Replace every `Arc<Mutex<Vec<WorldCommand>>>` in the codebase with this.
#[derive(Clone, Default)]
pub struct EngineQueue(pub Rc<RefCell<CommandBuffer>>);

impl EngineQueue {
    /// Push a typed command.  Costs one `borrow_mut()` (single flag check,
    /// never contended) and one `Vec::push` (amortised O(1)).
    #[inline]
    pub fn push(&self, cmd: EngineCommand) {
        self.0.borrow_mut().push(cmd);
    }

    /// Push an untyped closure.  Allocates one `Box`; use only for rare,
    /// complex mutations that have no matching variant.
    #[inline]
    pub fn push_raw<F>(&self, f: F)
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.0.borrow_mut().push(EngineCommand::Raw(Box::new(f)));
    }
}

/// Bevy resource that holds the shared command buffer.
///
/// Must only be accessed from the main thread.  The `Send`/`Sync` impls
/// are safe because Bevy's exclusive system scheduling guarantees that
/// `process_engine_queue` runs on the main thread with `&mut World`
/// access, and no other system can touch this resource concurrently.
pub struct EngineQueueResource(pub EngineQueue);

unsafe impl Send for EngineQueueResource {}
unsafe impl Sync for EngineQueueResource {}

impl Resource for EngineQueueResource {}

/// Exclusive Bevy system — drains and applies every queued command.
///
/// Runs once per frame, after Luau scripts have been ticked.
/// No lock, no allocation: just a Vec drain and a match dispatch.
pub fn process_engine_queue(world: &mut World) {
    let queue = world
        .get_resource::<EngineQueueResource>()
        .expect("EngineQueueResource missing")
        .0
        .0
        .clone();

    let mut buf = queue.borrow_mut();
    let commands: Vec<EngineCommand> = buf.drain(..).collect();
    drop(buf);

    for cmd in commands {
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
                    if let Some(mat) = world
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
        EngineCommand::SetEmissive { handle, r, g, b } => {
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Some(mat_h) = world
                    .get::<MeshMaterial3d<StandardMaterial>>(e)
                    .map(|m| m.0.clone())
                {
                    if let Some(mat) = world
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
                    tf.font_size = size;
                }
            }
        }
        EngineCommand::SetRigidBody { handle, dynamic } => {
            use bevy_rapier3d::dynamics::RigidBody;
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                if let Ok(mut em) = world.get_entity_mut(e) {
                    em.insert(if dynamic {
                        RigidBody::Dynamic
                    } else {
                        RigidBody::Fixed
                    });
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
                d.shadows_enabled = enabled;
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
                    e_mut.remove_parent_in_place();
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
        EngineCommand::LoadAsset { handle, asset_path } => {
            let handle_scene: Handle<Scene> = world.resource::<AssetServer>().load(&asset_path);
            if let Some(e) = world.resource::<HandleMap>().get_entity(handle) {
                world.entity_mut(e).insert(SceneRoot(handle_scene));
            }
        }
        EngineCommand::Raw(f) => f(world),
    }
}
