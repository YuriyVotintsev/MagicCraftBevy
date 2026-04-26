use bevy::prelude::*;

use crate::GameState;

#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
pub enum CombatPhase {
    #[default]
    Running,
    Paused,
    DevMenu,
}
