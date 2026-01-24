use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::stats::StatId;
use super::ids::{EffectTypeId, ParamId};
use super::expression::StatExpression;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamValueRaw {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Stat(StatId),
    Expr(StatExpression),
    Effect(Box<EffectDefRaw>),
    EffectList(Vec<EffectDefRaw>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDefRaw {
    pub effect_type: String,
    #[serde(default)]
    pub params: HashMap<String, ParamValueRaw>,
}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Stat(StatId),
    Expr(StatExpression),
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
}

impl ParamValue {
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_expr(&self) -> Option<&StatExpression> {
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
