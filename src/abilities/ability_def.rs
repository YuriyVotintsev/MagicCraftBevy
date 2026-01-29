use serde::{Deserialize, Serialize};

use super::ids::AbilityId;
use super::trigger_def::{TriggerDef, TriggerDefRaw};
use super::effect_def::{EffectDef, EffectDefRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AbilityDefRaw {
    pub id: String,
    pub trigger: TriggerDefRaw,
    pub effects: Vec<EffectDefRaw>,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub trigger: TriggerDef,
    pub effects: Vec<EffectDef>,
}
