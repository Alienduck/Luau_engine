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
pub fn process_button_interactions(world: &mut World) {
    let mut state = world.query::<(&Interaction, &mut PreviousInteraction, &LuauButtonSignals)>();
    let mut events = Vec::new();
    for (interaction, mut prev, signals) in state.iter_mut(world) {
        if *interaction != prev.0 {
            match (*interaction, prev.0) {
                (Interaction::Hovered, Interaction::None) => events.push(signals.enter),
                (Interaction::None, _) => events.push(signals.leave),
                (Interaction::Hovered, Interaction::Pressed) => events.push(signals.click),
                _ => {}
            }
            prev.0 = *interaction;
        }
    }
    let world_ptr = world as *mut World;
    let vm = world.non_send::<LuaVm>();
    vm.lua.set_app_data(world_ptr);

    for sig_id in events {
        let _ = LuaSignal { id: sig_id }.fire(&vm.lua, ());
    }

    vm.lua.remove_app_data::<*mut World>();
}
