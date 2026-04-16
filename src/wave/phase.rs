use bevy::prelude::*;

use crate::GameState;

#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(WavePhase = WavePhase::Combat)]
pub enum CombatPhase {
    #[default]
    Running,
    Paused,
    DevMenu,
}

#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
pub enum WavePhase {
    #[default]
    Combat,
    Shop,
}
