use bevy::{prelude::*, window::WindowResolution};

fn main() {
    println!("Hello Launcher!");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resize_constraints: WindowResizeConstraints {
                    min_height: 260.,
                    min_width: 480.,
                    ..default()
                },
                resolution: WindowResolution::new(720, 480),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(launcher_ui::LauncherPlugin)
        .run();
}
