use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    MobAI,
    Spawning,
    BlueprintActivation,
    BlueprintExecution,
    Damage,
    DamageApply,
    WaveManagement,
    Cleanup,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostGameSet;
