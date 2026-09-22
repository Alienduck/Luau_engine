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
            height: Val::Vh(25.),
            align_items: AlignItems::Center,
            overflow: Overflow::hidden(),
            border_radius: BorderRadius::all(Val::Px(10.)),
            ..default()
        },))
        .id();

    let image = commands
        .spawn((
            ImageNode::new(asset_server.load("images/banner.png")),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                aspect_ratio: Some(2560.0 / 640.0),
                padding: UiRect::horizontal(Val::Px(2.)).with_top(Val::Px(5.)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    let icon = commands
        .spawn((
            ImageNode::new(asset_server.load("images/lucid_white_icon.png")),
            Node {
                margin: UiRect::left(Val::Px(5.)),
                height: Val::Percent(80.),
                aspect_ratio: Some(159. / 121.),
                ..default()
            },
        ))
        .id();

    let title = commands
        .spawn((
            Text::new("Lucid Engine"),
            TextFont {
                font: asset_server.load("fonts/montserrat.ttf").into(),
                font_size: FontSize::Vw(5.),
                weight: FontWeight(700),
                ..default()
            },
            Node {
                padding: UiRect::all(Val::Px(5.)),
                ..default()
            },
        ))
        .id();

    commands.entity(container).add_child(image);
    commands.entity(container).add_child(icon);
    commands.entity(container).add_child(title);
}
