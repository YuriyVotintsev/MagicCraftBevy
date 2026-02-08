use std::collections::HashMap;
use serde::Deserialize;

use crate::abilities::expr::{ExprFamily, Raw, Resolved, ScalarExpr};
use crate::stats::StatRegistry;

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
pub struct StatesBlockRaw {
    pub initial: String,
    pub states: HashMap<String, StateDef<Raw>>,
}

#[derive(Debug, Clone)]
pub struct StatesBlock {
    pub initial: usize,
    pub states: Vec<StateDef>,
    pub state_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(bound(deserialize = "ScalarExpr<F>: Deserialize<'de>, F::StatRef: Deserialize<'de>, F::ComponentDef: Deserialize<'de>, F::StatesBlock: Deserialize<'de>"))]
pub struct EntityDef<F: ExprFamily = Resolved> {
    #[serde(default)]
    pub count: Option<ScalarExpr<F>>,
    #[serde(default)]
    pub base_stats: HashMap<F::StatRef, f32>,
    #[serde(default)]
    pub abilities: Vec<String>,
    pub components: Vec<F::ComponentDef>,
    #[serde(default)]
    pub states: Option<F::StatesBlock>,
}

impl StateDef<Raw> {
    pub fn resolve(&self, stat_registry: &StatRegistry, state_indices: &HashMap<String, usize>) -> StateDef {
        StateDef {
            components: self.components.iter().map(|c| c.resolve(stat_registry, Some(state_indices))).collect(),
            transitions: self.transitions.iter().map(|c| c.resolve(stat_registry, Some(state_indices))).collect(),
        }
    }
}

impl StatesBlockRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> StatesBlock {
        let state_names: Vec<String> = self.states.keys().cloned().collect();
        let state_indices: HashMap<String, usize> = state_names.iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        let initial = *state_indices.get(&self.initial)
            .unwrap_or_else(|| panic!("unknown initial state '{}'", self.initial));

        let states = state_names.iter()
            .map(|name| self.states[name].resolve(stat_registry, &state_indices))
            .collect();

        StatesBlock { initial, states, state_names }
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
            components: self.components.iter().map(|c| c.resolve(stat_registry, None)).collect(),
            states: self.states.as_ref().map(|s| s.resolve(stat_registry)),
        }
    }
}
