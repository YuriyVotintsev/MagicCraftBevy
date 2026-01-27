use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    MobAI,
    AbilityActivation,
    AbilityExecution,
    Damage,
    WaveManagement,
}
