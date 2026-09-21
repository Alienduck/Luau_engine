use bevy::prelude::*;

pub(crate) struct TopPanel;

impl Plugin for TopPanel {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build);
    }
}

fn build(mut commands: Commands, asset_server: Res<AssetServer>) {
    let container = commands
        .spawn(Node {
            width: percent(100),
            height: percent(20),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .id();

    let image = commands
        .spawn((
            ImageNode::new(asset_server.load("images/banner.png"))
                .with_mode(NodeImageMode::Stretch),
            Node {
                width: percent(100),
                aspect_ratio: Some(16.0 / 9.0),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.entity(container).add_child(image);
}
