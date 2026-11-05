use bevy::prelude::*;

pub fn bottom_panel() -> impl Scene {
    bsn! {
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
        }
    }
}
