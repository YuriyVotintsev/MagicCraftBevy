use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Clone, Deserialize)]
pub struct GameBalance {
    pub wave: WaveBalance,
    pub arena: ArenaBalance,
    pub shop: ShopBalance,
}

#[derive(Clone, Deserialize)]
pub struct WaveBalance {
    pub base_enemies: u32,
    pub enemies_per_wave: u32,
    pub base_concurrent: u32,
    pub concurrent_per_wave: u32,
    pub spawn_threshold: u32,
    pub reward: u32,
    pub shop_delay: f32,
    pub marker_duration: f32,
}

#[derive(Clone, Deserialize)]
pub struct ArenaBalance {
    pub width: f32,
    pub height: f32,
    pub half_w: f32,
    pub half_h: f32,
    pub corner_radius: f32,
}

#[derive(Clone, Deserialize)]
pub struct ShopBalance {
    pub artifact_slots: usize,
    pub offerings_count: usize,
    pub base_reroll_cost: u32,
    pub sell_price_percent: u32,
}
