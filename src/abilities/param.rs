use serde::{Deserialize, Serialize};

use crate::stats::{ComputedStats, Expression, StatId};
use super::ids::NodeDefId;
use super::node::NodeDefRaw;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ParamValueRaw {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Stat(String),
    Action(Box<NodeDefRaw>),
    ActionList(Vec<NodeDefRaw>),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Stat(StatId),
    Expr(Expression),
    Action(NodeDefId),
    ActionList(Vec<NodeDefId>),
}

impl ParamValue {
    pub fn evaluate_f32(&self, stats: &ComputedStats) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f32),
            Self::Stat(stat_id) => Some(stats.get(*stat_id)),
            Self::Expr(expr) => Some(expr.evaluate_computed(stats)),
            _ => None,
        }
    }

    pub fn evaluate_i32(&self, stats: &ComputedStats) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Float(v) => Some(*v as i32),
            Self::Stat(stat_id) => Some(stats.get(*stat_id) as i32),
            Self::Expr(expr) => Some(expr.evaluate_computed(stats) as i32),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_expr(&self) -> Option<&Expression> {
        match self {
            Self::Expr(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_action_list(&self) -> Option<&Vec<NodeDefId>> {
        match self {
            Self::ActionList(v) => Some(v),
            _ => None,
        }
    }
}
