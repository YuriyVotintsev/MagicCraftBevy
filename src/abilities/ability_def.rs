use std::sync::Arc;
use serde::{Deserialize, Serialize};

use super::ids::AbilityId;
use super::trigger_def::{TriggerDef, TriggerDefRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AbilityDefRaw {
    pub id: String,
    pub trigger: TriggerDefRaw,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub trigger: Arc<TriggerDef>,
}
