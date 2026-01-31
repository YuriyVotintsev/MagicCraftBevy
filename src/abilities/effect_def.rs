use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::stats::{ComputedStats, Expression, StatId};
use super::ids::{ActionTypeId, ParamId};
use super::trigger_def::ActionDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ParamValueRaw {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Stat(String),
    Action(Box<ActionDefRaw>),
    ActionList(Vec<ActionDefRaw>),
}

use super::trigger_def::ActionDefRaw;

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
    Action(Arc<ActionDef>),
    ActionList(Vec<Arc<ActionDef>>),
}

#[derive(Debug, Clone)]
pub struct EffectDef {
    pub effect_type: ActionTypeId,
    pub params: HashMap<ParamId, ParamValue>,
}

impl ActionDef {
    pub fn get_param<'a>(&'a self, name: &str, registry: &crate::abilities::registry::ActionRegistry) -> Option<&'a ParamValue> {
        let id = registry.get_param_id(name)?;
        self.params.get(&id)
    }

    pub fn get_action_list<'a>(&'a self, name: &str, registry: &crate::abilities::registry::ActionRegistry) -> Option<&'a Vec<Arc<ActionDef>>> {
        self.get_param(name, registry)?.as_action_list()
    }

    pub fn get_f32(
        &self,
        name: &str,
        stats: &ComputedStats,
        registry: &crate::abilities::registry::ActionRegistry
    ) -> Option<f32> {
        self.get_param(name, registry)?.evaluate_f32(stats)
    }

    pub fn get_i32(
        &self,
        name: &str,
        stats: &ComputedStats,
        registry: &crate::abilities::registry::ActionRegistry
    ) -> Option<i32> {
        self.get_param(name, registry)?.evaluate_i32(stats)
    }
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

    pub fn as_action_list(&self) -> Option<&Vec<Arc<ActionDef>>> {
        match self {
            Self::ActionList(v) => Some(v),
            _ => None,
        }
    }
}
