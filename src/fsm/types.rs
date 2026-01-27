use serde::Deserialize;
use std::collections::HashMap;

use crate::physics::ColliderDef;

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

fn default_cooldown() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub enum BehaviourDef {
    MoveTowardPlayer,
    UseAbilities {
        abilities: Vec<String>,
        #[serde(default = "default_cooldown")]
        cooldown: f32,
    },
    KeepDistance {
        min: f32,
        max: f32,
    },
}

impl BehaviourDef {
    pub fn type_name(&self) -> &'static str {
        match self {
            BehaviourDef::MoveTowardPlayer => "MoveTowardPlayer",
            BehaviourDef::UseAbilities { .. } => "UseAbilities",
            BehaviourDef::KeepDistance { .. } => "KeepDistance",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum TransitionDef {
    WhenNear(Vec<(String, f32)>),
    AfterTime(String, f32),
}

impl TransitionDef {
    pub fn type_name(&self) -> &'static str {
        match self {
            TransitionDef::WhenNear(_) => "WhenNear",
            TransitionDef::AfterTime(_, _) => "AfterTime",
        }
    }
}
