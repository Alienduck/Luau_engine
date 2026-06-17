use crate::types::signal::LuaSignal;
use bevy::prelude::*;
use luau_runtime::vm::LuaVm;

/// Bevy component which handles the Luau signals IDs for the button
#[derive(Component, Clone)]
pub struct LuauButtonSignals {
    pub click: u64,
    pub enter: u64,
    pub leave: u64,
}

/// Keep in memory the previous state before click (Pressed -> Hovered)
#[derive(Component, Default)]
pub struct PreviousInteraction(pub Interaction);

/// Bevy system which listen the mouse interactions on the UI
pub fn process_button_interactions(
    mut query: Query<
        (&Interaction, &mut PreviousInteraction, &LuauButtonSignals),
        Changed<Interaction>,
    >,
    vm: NonSend<LuaVm>,
) {
    for (interaction, mut prev, signals) in query.iter_mut() {
        match (*interaction, prev.0) {
            (Interaction::Hovered, Interaction::None) => {
                let _ = LuaSignal { id: signals.enter }.fire(&vm.lua, ());
            }
            (Interaction::None, _) => {
                let _ = LuaSignal { id: signals.leave }.fire(&vm.lua, ());
            }
            (Interaction::Hovered, Interaction::Pressed) => {
                let _ = LuaSignal { id: signals.click }.fire(&vm.lua, ());
            }
            _ => {}
        }
        prev.0 = *interaction;
    }
}
