use serde::{Deserialize, Serialize};

use super::ids::AbilityId;
use super::activator_def::{ActivatorDef, ActivatorDefRaw};
use super::effect_def::{EffectDef, EffectDefRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AbilityDefRaw {
    pub id: String,
    pub activator: ActivatorDefRaw,
    pub effects: Vec<EffectDefRaw>,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub activator: ActivatorDef,
    pub effects: Vec<EffectDef>,
}
