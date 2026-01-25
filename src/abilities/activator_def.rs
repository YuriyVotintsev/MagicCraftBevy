use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::ids::{ActivatorTypeId, ParamId};
use super::effect_def::{ParamValue, ParamValueRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ActivatorDefRaw {
    pub activator_type: String,
    #[serde(default)]
    pub params: HashMap<String, ParamValueRaw>,
}

#[derive(Debug, Clone)]
pub struct ActivatorDef {
    pub activator_type: ActivatorTypeId,
    #[allow(dead_code)]
    pub params: HashMap<ParamId, ParamValue>,
}

#[derive(Debug, Clone, Default)]
pub struct ActivatorState {
    #[allow(dead_code)]
    pub params: HashMap<ParamId, f32>,
}

#[allow(dead_code)]
impl ActivatorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: ParamId) -> f32 {
        self.params.get(&key).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, key: ParamId, value: f32) {
        self.params.insert(key, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationResult {
    NotReady,
    Ready,
}
