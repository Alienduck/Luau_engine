use bevy::prelude::*;

/// Thin wrapper around a Bevy [`App`] with engine-level defaults baked in.
///
/// Spawns a window titled "Luau Engine" and applies [`DefaultPlugins`].
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
