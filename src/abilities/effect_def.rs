use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::stats::{Expression, StatId};
use super::ids::{EffectTypeId, ParamId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ParamValueRaw {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Stat(String),
    Effect(Box<EffectDefRaw>),
    EffectList(Vec<EffectDefRaw>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct EffectDefRaw {
    pub effect_type: String,
    #[serde(default)]
    pub params: HashMap<String, ParamValueRaw>,
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
    Effect(Box<EffectDef>),
    EffectList(Vec<EffectDef>),
}

#[derive(Debug, Clone)]
pub struct EffectDef {
    pub effect_type: EffectTypeId,
    pub params: HashMap<ParamId, ParamValue>,
}

impl EffectDef {
    pub fn get_param<'a>(&'a self, name: &str, registry: &crate::abilities::registry::EffectRegistry) -> Option<&'a ParamValue> {
        let id = registry.get_param_id(name)?;
        self.params.get(&id)
    }

    pub fn get_f32(&self, name: &str, stats: &ComputedStats, registry: &crate::abilities::registry::EffectRegistry) -> Option<f32> {
        self.get_param(name, registry)?.evaluate_f32(stats)
    }

    pub fn get_i32(&self, name: &str, stats: &ComputedStats, registry: &crate::abilities::registry::EffectRegistry) -> Option<i32> {
        self.get_param(name, registry)?.evaluate_i32(stats)
    }

    pub fn get_effect_list<'a>(&'a self, name: &str, registry: &crate::abilities::registry::EffectRegistry) -> Option<&'a Vec<EffectDef>> {
        self.get_param(name, registry)?.as_effect_list()
    }
}

use crate::stats::ComputedStats;

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

    pub fn as_effect_list(&self) -> Option<&Vec<EffectDef>> {
        match self {
            Self::EffectList(v) => Some(v),
            _ => None,
        }
    }
}
