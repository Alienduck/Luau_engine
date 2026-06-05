use bevy::prelude::*;

/// Thin wrapper around a Bevy App with engine-level defaults.
pub struct EngineApp(pub App);

impl EngineApp {
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luau Engine".into(),
                ..default()
            }),
            ..default()
        }));
        Self(app)
    }

    pub fn inner(&mut self) -> &mut App {
        &mut self.0
    }

    pub fn run(&mut self) {
        self.0.run();
    }
}

impl Default for EngineApp {
    fn default() -> Self {
        Self::new()
    }
}

/// An object to compute the Gui Anchor point when the window is resized
#[derive(Component, Default)]
pub struct GuiObjectData {
    pub anchor_point: Vec2,
}
