use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    MobAI,
    Spawning,
    BlueprintActivation,
    BlueprintExecution,
    BlueprintLifecycle,
    Damage,
    DamageApply,
    WaveManagement,
    Cleanup,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShopSet {
    Input,
    Process,
    Display,
}
