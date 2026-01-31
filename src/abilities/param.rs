use serde::{Deserialize, Serialize};

use crate::stats::{ComputedStats, StatId};
use super::node::NodeDefRaw;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamValueRaw {
    Float(f32),
    Int(i32),
    Stat(String),
    Action(Box<NodeDefRaw>),
    ActionList(Vec<NodeDefRaw>),
}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Stat(StatId),
}

impl ParamValue {
    pub fn evaluate_f32(&self, stats: &ComputedStats) -> f32 {
        match self {
            Self::Float(v) => *v,
            Self::Int(v) => *v as f32,
            Self::Stat(stat_id) => stats.get(*stat_id),
        }
    }

    pub fn evaluate_i32(&self, stats: &ComputedStats) -> i32 {
        match self {
            Self::Int(v) => *v,
            Self::Float(v) => *v as i32,
            Self::Stat(stat_id) => stats.get(*stat_id) as i32,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(_) | Self::Stat(_) => None,
        }
    }
}
