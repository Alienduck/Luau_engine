use bevy::prelude::*;

pub(crate) struct TopPanel;

impl Plugin for TopPanel {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build);
    }
}

fn build(mut commands: Commands) {
    commands.spawn((Node { ..default() }));
}
