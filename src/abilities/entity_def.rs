use std::collections::HashMap;
use serde::Deserialize;

use crate::abilities::expr::{ExprFamily, Raw, Resolved, ScalarExpr};
use crate::stats::StatRegistry;

#[allow(dead_code)]
pub type StateDefRaw = StateDef<Raw>;
#[allow(dead_code)]
pub type StatesBlockRaw = StatesBlock<Raw>;
pub type EntityDefRaw = EntityDef<Raw>;

#[derive(Debug, Clone, Deserialize)]
#[serde(bound(deserialize = "F::ComponentDef: Deserialize<'de>"))]
pub struct StateDef<F: ExprFamily = Resolved> {
    #[serde(default)]
    pub components: Vec<F::ComponentDef>,
    #[serde(default)]
    pub transitions: Vec<F::ComponentDef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(bound(deserialize = "StateDef<F>: Deserialize<'de>"))]
pub struct StatesBlock<F: ExprFamily = Resolved> {
    pub initial: String,
    pub states: HashMap<String, StateDef<F>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(bound(deserialize = "ScalarExpr<F>: Deserialize<'de>, F::StatRef: Deserialize<'de>, F::ComponentDef: Deserialize<'de>, StatesBlock<F>: Deserialize<'de>"))]
pub struct EntityDef<F: ExprFamily = Resolved> {
    #[serde(default)]
    pub count: Option<ScalarExpr<F>>,
    #[serde(default)]
    pub base_stats: HashMap<F::StatRef, f32>,
    #[serde(default)]
    pub abilities: Vec<String>,
    pub components: Vec<F::ComponentDef>,
    #[serde(default)]
    pub states: Option<StatesBlock<F>>,
}

impl StateDef<Raw> {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> StateDef {
        StateDef {
            components: self.components.iter().map(|c| c.resolve(stat_registry)).collect(),
            transitions: self.transitions.iter().map(|c| c.resolve(stat_registry)).collect(),
        }
    }
}

impl StatesBlock<Raw> {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> StatesBlock {
        StatesBlock {
            initial: self.initial.clone(),
            states: self.states.iter().map(|(k, v)| (k.clone(), v.resolve(stat_registry))).collect(),
        }
    }
}

impl EntityDef<Raw> {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> EntityDef {
        EntityDef {
            count: self.count.as_ref().map(|c| c.resolve(stat_registry)),
            base_stats: self.base_stats.iter()
                .filter_map(|(name, value)| {
                    stat_registry.get(name).map(|id| (id, *value))
                })
                .collect(),
            abilities: self.abilities.clone(),
            components: self.components.iter().map(|c| c.resolve(stat_registry)).collect(),
            states: self.states.as_ref().map(|s| s.resolve(stat_registry)),
        }
    }
}
