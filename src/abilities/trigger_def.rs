use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use super::ids::{TriggerTypeId, ActionTypeId, ParamId};
use super::effect_def::{ParamValue, ParamValueRaw};

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
    pub triggers: Vec<Arc<TriggerDef>>,
}

#[derive(Debug, Clone)]
pub struct TriggerDef {
    pub trigger_type: TriggerTypeId,
    pub params: HashMap<ParamId, ParamValue>,
    pub actions: Vec<Arc<ActionDef>>,
}
