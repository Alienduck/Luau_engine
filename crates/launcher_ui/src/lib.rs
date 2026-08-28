use bevy::prelude::*;

mod top_panel;

pub struct LauncherPlugin;

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(top_panel::TopPanel)
            .add_systems(Startup, (setup, setup_banner));
    }
}

#[derive(Resource, Default)]
pub struct BannerImage(pub Handle<Image>);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn setup_banner(mut commands: Commands, asset_server: Res<AssetServer>) {
    let banner_handle = asset_server.load("images/banner.png");
    commands.insert_resource(BannerImage(banner_handle));
}
