use bevy::prelude::*;

pub(crate) struct TopPanel;

impl Plugin for TopPanel {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build);
    }
}

fn build(mut commands: Commands, asset_server: Res<AssetServer>) {
    let container = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .id();

    let image = commands
        .spawn((
            ImageNode::new(asset_server.load("images/banner.png")),
            Node {
                width: Val::Percent(100.0),
                aspect_ratio: Some(2560.0 / 640.0),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    commands.entity(container).add_child(image);
}
