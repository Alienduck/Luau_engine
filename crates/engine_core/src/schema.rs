use bevy_tnua::prelude::*;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum CharacterControllerScheme {
    Jumping(TnuaBuiltinJump),
}
