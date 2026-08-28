use bevy::prelude::*;

fn main() {
    println!("Hello Launcher!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(launcher_ui::LauncherPlugin)
        .run();
}
