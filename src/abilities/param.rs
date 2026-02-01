use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::stats::{ComputedStats, Expression, ExpressionRaw, Modifiers, StatId, StatRegistry};

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
}

pub trait ParseNodeParams: Send + Sync + 'static {
    fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NoParams;

impl ParseNodeParams for NoParams {
    fn parse(_: &HashMap<String, ParamValueRaw>, _: &StatRegistry) -> Self {
        Self
    }
}

pub fn resolve_param_value(raw: &ParamValueRaw, stat_registry: &StatRegistry) -> ParamValue {
    match raw {
        ParamValueRaw::Float(v) => ParamValue::Float(*v),
        ParamValueRaw::Int(v) => ParamValue::Int(*v),
        ParamValueRaw::Bool(v) => ParamValue::Bool(*v),
        ParamValueRaw::Stat(name) => {
            let stat_id = stat_registry.get(name).unwrap_or_else(|| panic!("Unknown stat '{}'", name));
            ParamValue::Stat(stat_id)
        }
        ParamValueRaw::Expr(expr) => ParamValue::Expr(expr.resolve(stat_registry)),
    }
}
