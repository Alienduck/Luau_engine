use bevy::prelude::*;

mod top_panel;

pub struct LauncherPlugin;

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup.spawn(), setup_banner));
    }
}

#[derive(Resource, Default)]
pub struct BannerImage(pub Handle<Image>);

fn setup() -> impl SceneList {
    bsn_list![Camera2d, top_panel::top_panel()]
}

pub fn setup_banner(mut commands: Commands, asset_server: Res<AssetServer>) {
    let banner_handle = asset_server.load("images/banner.png");
    commands.insert_resource(BannerImage(banner_handle));
}
