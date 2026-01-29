use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::ids::{TriggerTypeId, ParamId};
use super::effect_def::{ParamValue, ParamValueRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDefRaw {
    pub trigger_type: String,
    #[serde(default)]
    pub params: HashMap<String, ParamValueRaw>,
}

#[derive(Debug, Clone)]
pub struct TriggerDef {
    pub trigger_type: TriggerTypeId,
    pub params: HashMap<ParamId, ParamValue>,
}
