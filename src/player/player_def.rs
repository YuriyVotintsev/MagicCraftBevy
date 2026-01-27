use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

use crate::fsm::{ColliderDef, ColliderShape};

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerDef {
    pub visual: PlayerVisualDef,
    #[serde(default = "default_player_collider")]
    pub collider: ColliderDef,
    pub base_stats: HashMap<String, f32>,
    #[allow(dead_code)]
    pub abilities: Vec<String>,
}

fn default_player_collider() -> ColliderDef {
    ColliderDef {
        shape: ColliderShape::Rectangle,
        size: 100.0,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerVisualDef {
    pub size: f32,
    pub color: [f32; 3],
}

pub fn load_player_def(path: &str) -> PlayerDef {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("Failed to read player definition file '{}': {}", path, e)
    });
    ron::from_str(&content).unwrap_or_else(|e| {
        panic!(
            "Failed to parse player definition '{}': {}\nContent:\n{}",
            path, e, content
        )
    })
}
