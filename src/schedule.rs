use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    MobAI,
    Spawning,
    AbilityActivation,
    AbilityExecution,
    Damage,
    WaveManagement,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostGameSet;
