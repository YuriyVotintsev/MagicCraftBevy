use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct MobDef {
    pub name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub abilities: Vec<String>,
    pub visual: VisualDef,
    #[serde(default)]
    pub collider: ColliderDef,
    #[serde(default)]
    pub base_stats: HashMap<String, f32>,
    pub initial_state: String,
    pub states: HashMap<String, StateDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VisualDef {
    pub shape: Shape,
    pub size: f32,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColliderDef {
    pub shape: ColliderShape,
    pub size: f32,
}

impl Default for ColliderDef {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Circle,
            size: 30.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum ColliderShape {
    #[default]
    Circle,
    Rectangle,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Shape {
    Circle,
    Rectangle,
    Triangle,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateDef {
    #[serde(default)]
    pub behaviour: Vec<BehaviourDef>,
    #[serde(default)]
    pub transitions: Vec<TransitionDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum BehaviourDef {
    MoveTowardPlayer,
    UseAbilities(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
pub enum TransitionDef {
    WhenNear(Vec<(String, f32)>),
    AfterTime(String, f32),
}
