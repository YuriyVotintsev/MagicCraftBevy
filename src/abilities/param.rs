use serde::{Deserialize, Serialize};

use crate::stats::{ComputedStats, Expression, ExpressionRaw, Modifiers, StatId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamValueRaw {
    Float(f32),
    Int(i32),
    Bool(bool),
    Stat(String),
    Expr(ExpressionRaw),
}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Stat(StatId),
    Expr(Expression),
}

impl ParamValue {
    pub fn evaluate_f32(&self, stats: &ComputedStats) -> f32 {
        match self {
            Self::Float(v) => *v,
            Self::Int(v) => *v as f32,
            Self::Bool(v) => if *v { 1.0 } else { 0.0 },
            Self::Stat(stat_id) => stats.get(*stat_id),
            Self::Expr(expr) => expr.evaluate(&Modifiers::default(), stats),
        }
    }

    pub fn evaluate_i32(&self, stats: &ComputedStats) -> i32 {
        match self {
            Self::Int(v) => *v,
            Self::Float(v) => *v as i32,
            Self::Bool(v) => *v as i32,
            Self::Stat(stat_id) => stats.get(*stat_id) as i32,
            Self::Expr(expr) => expr.evaluate(&Modifiers::default(), stats) as i32,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Int(v) => *v != 0,
            Self::Float(v) => *v != 0.0,
            Self::Stat(_) | Self::Expr(_) => false,
        }
    }
}
