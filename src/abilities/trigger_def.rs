use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::ids::{TriggerTypeId, ActionTypeId, ParamId, ActionDefId, TriggerDefId};
use super::param::{ParamValue, ParamValueRaw};
use crate::stats::ComputedStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionDefRaw {
    Full(String, HashMap<String, ParamValueRaw>, Vec<TriggerDefRaw>),
    NoTriggers(String, HashMap<String, ParamValueRaw>),
    NoParams(String, Vec<TriggerDefRaw>),
    OnlyName(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TriggerDefRaw {
    Full(String, HashMap<String, ParamValueRaw>, Vec<ActionDefRaw>),
    NoActions(String, HashMap<String, ParamValueRaw>),
    NoParams(String, Vec<ActionDefRaw>),
    OnlyName(String),
}

#[derive(Debug, Clone)]
pub struct ActionDef {
    pub action_type: ActionTypeId,
    pub params: HashMap<ParamId, ParamValue>,
    pub triggers: Vec<TriggerDefId>,
}

#[derive(Debug, Clone)]
pub struct TriggerDef {
    pub trigger_type: TriggerTypeId,
    pub params: HashMap<ParamId, ParamValue>,
    pub actions: Vec<ActionDefId>,
}

impl ActionDef {
    pub fn get_param<'a>(&'a self, name: &str, registry: &crate::abilities::registry::ActionRegistry) -> Option<&'a ParamValue> {
        let id = registry.get_param_id(name)?;
        self.params.get(&id)
    }

    pub fn get_action_list<'a>(&'a self, name: &str, registry: &crate::abilities::registry::ActionRegistry) -> Option<&'a Vec<ActionDefId>> {
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
