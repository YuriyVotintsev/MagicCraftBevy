use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

#[derive(Asset, Resource, TypePath, Clone, Deserialize)]
pub struct GameBalance {
    pub wave: WaveBalance,
    pub arena: ArenaBalance,
    pub run: RunBalance,
}

#[derive(Clone, Deserialize)]
pub struct WaveBalance {
    pub start_enemies: u32,
    pub max_enemies: u32,
    pub ramp_duration_secs: f32,
    pub safe_spawn_radius: f32,
    pub shop_delay: f32,
}

#[derive(Clone, Deserialize)]
pub struct ArenaBalance {
    pub start_width: f32,
    pub start_height: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Deserialize)]
pub struct RunBalance {
    pub coins_per_kill: u32,
    pub hp_scale_per_sec: f32,
    pub dmg_scale_per_sec: f32,
    pub coin_attraction_duration: f32,
}
