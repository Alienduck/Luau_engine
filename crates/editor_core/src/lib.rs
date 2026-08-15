use bevy::prelude::*;

#[derive(Resource, Default)]
pub enum AppState {
    #[default]
    Edit,
    Play,
}
