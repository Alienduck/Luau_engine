use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::prelude::*;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum ControlerScheme {
    Jump(TnuaBuiltinJump),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_plugins((
            TnuaControllerPlugin::<ControlerScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .init_resource::<GizmoConfigStore>()
        .add_systems(Startup, (setup_environment, setup_player).chain())
        .add_systems(Update, apply_controls.in_set(TnuaUserControlsSystems))
        .run();
}

fn setup_environment(asset_server: Res<AssetServer>, mut commands: Commands) {
    let model: Handle<WorldAsset> =
        asset_server.load("models/stylized_classroom/scene.gltf#Scene0");
    commands.spawn((
        Transform::default(),
        WorldAssetRoot(model),
        ColliderConstructorHierarchy {
            default_constructor: Some(ColliderConstructor::TrimeshFromMeshWithConfig(
                TrimeshFlags::all(),
            )),
            ..default()
        },
        Friction::new(0.0),
        Restitution::new(0.0),
        CollisionEventsEnabled,
        SweptCcd::default(),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((PointLight::default(), Transform::from_xyz(5.0, 8.0, 5.0)));
}

fn setup_player(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut controler_scheme_config: ResMut<Assets<ControlerSchemeConfig>>,
) {
    commands.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        Mesh3d(mesh.add(Capsule3d::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Collider::capsule(0.5, 0.5),
        RigidBody::Dynamic,
        GravityScale(3.0),
        TnuaController::<ControlerScheme>::default(),
        TnuaConfig::<ControlerScheme>(controler_scheme_config.add(ControlerSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                speed: 16.0,
                float_height: 1.0,
                ..default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 24.0,
                ..default()
            },
        })),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
        LockedAxes::ROTATION_LOCKED,
    ));
}

fn apply_controls(
    mut tnua_query: Query<&mut TnuaController<ControlerScheme>>,
    mut gizmo_store: ResMut<GizmoConfigStore>,
    inputs: ResMut<ButtonInput<KeyCode>>,
) {
    let Ok(mut ctrl) = tnua_query.single_mut() else {
        return;
    };
    ctrl.initiate_action_feeding();

    let mut direction = Vec3::ZERO;

    if inputs.pressed(KeyCode::KeyW) {
        direction += Vec3::NEG_Z;
    }
    if inputs.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }
    if inputs.pressed(KeyCode::KeyA) {
        direction += Vec3::NEG_X;
    }
    if inputs.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }
    if inputs.pressed(KeyCode::KeyR) {
        let gizmos = gizmo_store.config_mut::<PhysicsGizmos>().0;
        gizmos.enabled = !gizmos.enabled;
    }
    ctrl.basis = TnuaBuiltinWalk {
        desired_motion: direction.normalize_or_zero(),
        ..default()
    };
    if inputs.pressed(KeyCode::Space) {
        ctrl.action(ControlerScheme::Jump(Default::default()));
    }
}
