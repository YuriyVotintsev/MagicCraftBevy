use serde::{Deserialize, Serialize};

use super::ids::{AbilityId, TriggerDefId, ActionDefId, TriggerTypeId};
use super::trigger_def::{TriggerDef, TriggerDefRaw, ActionDef};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AbilityDefRaw {
    pub id: String,
    pub trigger: TriggerDefRaw,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub root_trigger: TriggerDefId,

    triggers: Vec<TriggerDef>,
    actions: Vec<ActionDef>,
}

impl AbilityDef {
    pub fn get_trigger(&self, id: TriggerDefId) -> Option<&TriggerDef> {
        self.triggers.get(id.0 as usize)
    }

    pub fn get_action(&self, id: ActionDefId) -> Option<&ActionDef> {
        self.actions.get(id.0 as usize)
    }

    pub fn new(id: AbilityId) -> Self {
        Self {
            id,
            root_trigger: TriggerDefId(0),
            triggers: vec![],
            actions: vec![],
        }
    }

    pub fn add_action(&mut self, def: ActionDef) -> ActionDefId {
        let id = ActionDefId(self.actions.len() as u32);
        self.actions.push(def);
        id
    }

    pub fn add_trigger(&mut self, def: TriggerDef) -> TriggerDefId {
        let id = TriggerDefId(self.triggers.len() as u32);
        self.triggers.push(def);
        id
    }

    pub fn set_root_trigger(&mut self, id: TriggerDefId) {
        self.root_trigger = id;
    }

    pub fn has_trigger(&self, action_id: ActionDefId, trigger_type: TriggerTypeId) -> bool {
        let Some(action) = self.get_action(action_id) else {
            return false;
        };

        action.triggers.iter().any(|&trigger_id| {
            self.get_trigger(trigger_id)
                .map(|t| t.trigger_type == trigger_type)
                .unwrap_or(false)
        })
    }
}
