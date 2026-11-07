use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_systems(Startup, setup_scene)
        // Chaque insertion dans un système Update SÉPARÉ, sur plusieurs frames,
        // pour reproduire fidèlement le pattern "Collider.Parent = subject" puis
        // "Rigidbody.Parent = subject" puis "CharacterController.Parent = subject"
        // exécutés par des closures push_raw distinctes à des frames différentes.
        .add_systems(Update, insert_collider.run_if(on_frame(1)))
        .add_systems(Update, insert_rigidbody.run_if(on_frame(2)))
        .add_systems(Update, insert_tnua.run_if(on_frame(3)))
        .add_systems(Update, apply_controls.in_set(TnuaUserControlsSystems))
        .run();
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum ControlScheme {
    #[allow(dead_code)]
    Jump(TnuaBuiltinJump),
}

#[derive(Resource)]
struct Player(Entity);

fn on_frame(n: u32) -> impl Fn(Local<u32>) -> bool {
    move |mut counter: Local<u32>| {
        *counter += 1;
        *counter == n
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((PointLight::default(), Transform::from_xyz(5.0, 8.0, 5.0)));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));
    for (pos, size) in [
        (Vec3::new(0.0, 2.0, -4.0), Vec3::new(8.0, 4.0, 0.5)),
        (Vec3::new(-4.0, 2.0, 0.0), Vec3::new(0.5, 4.0, 8.0)),
        (Vec3::new(4.0, 2.0, 0.0), Vec3::new(0.5, 4.0, 8.0)),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(Color::from(css::GRAY))),
            Transform::from_translation(pos),
            RigidBody::Static,
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
        ));
    }

    // Entité "subject" spawnée quasi-vide, comme Part.new()
    let subject = commands
        .spawn((
            Transform::from_xyz(0.0, 4.0, 10.0),
            Mesh3d(meshes.add(Capsule3d {
                radius: 0.5,
                half_length: 0.5,
            })),
            MeshMaterial3d(materials.add(Color::from(css::DARK_CYAN))),
        ))
        .id();
    commands.insert_resource(Player(subject));
}

// Frame 1 : comme s_collider.Parent = subject
fn insert_collider(mut commands: Commands, player: Res<Player>) {
    commands
        .entity(player.0)
        .insert(Collider::capsule(0.5, 1.0));
}

// Frame 2 : comme rb_subject.Parent = subject
fn insert_rigidbody(mut commands: Commands, player: Res<Player>) {
    commands.entity(player.0).insert((
        RigidBody::Dynamic,
        LinearVelocity(Vec3::new(0.0, 0.0, -25.0)),
    ));
}

// Frame 3 : comme controller.Parent = subject
fn insert_tnua(
    mut commands: Commands,
    player: Res<Player>,
    mut configs: ResMut<Assets<ControlSchemeConfig>>,
) {
    commands.entity(player.0).insert((
        TnuaController::<ControlScheme>::default(),
        TnuaConfig::<ControlScheme>(configs.add(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                speed: 5.0,
                float_height: 1.5,
                ..default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 4.0,
                ..default()
            },
        })),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        LockedAxes::ROTATION_LOCKED,
    ));
}

fn apply_controls(mut query: Query<&mut TnuaController<ControlScheme>>) {
    if let Ok(mut controller) = query.single_mut() {
        controller.initiate_action_feeding();
        controller.basis = TnuaBuiltinWalk::default();
    }
}
