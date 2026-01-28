use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::ids::{ActivatorTypeId, ParamId};
use super::effect_def::{ParamValue, ParamValueRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatorDefRaw {
    pub activator_type: String,
    #[serde(default)]
    pub params: HashMap<String, ParamValueRaw>,
}

#[derive(Debug, Clone)]
pub struct ActivatorDef {
    pub activator_type: ActivatorTypeId,
    pub params: HashMap<ParamId, ParamValue>,
}
