use bevy::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luau Engine".to_string(),
                ..default()
            }),
            ..default()
        }))
        .run();
}
