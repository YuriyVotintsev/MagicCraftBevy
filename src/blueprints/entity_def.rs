use std::collections::HashMap;
use serde::Deserialize;

use crate::blueprints::expr::{ScalarExpr, ScalarExprRaw};
use crate::stats::{StatId, StatRegistry};

use super::components::{ComponentDef, ComponentDefRaw};

#[derive(Debug, Clone, Deserialize)]
pub struct StateDefRaw {
    #[serde(default)]
    pub components: Vec<ComponentDefRaw>,
    #[serde(default)]
    pub transitions: Vec<ComponentDefRaw>,
}

#[derive(Debug, Clone)]
pub struct StateDef {
    pub components: Vec<ComponentDef>,
    pub transitions: Vec<ComponentDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatesBlockRaw {
    pub initial: String,
    pub states: HashMap<String, StateDefRaw>,
}

#[derive(Debug, Clone)]
pub struct StatesBlock {
    pub initial: usize,
    pub states: Vec<StateDef>,
    pub state_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntityDefRaw {
    #[serde(default)]
    pub count: Option<ScalarExprRaw>,
    #[serde(default)]
    pub base_stats: HashMap<String, f32>,
    #[serde(default)]
    pub abilities: Vec<String>,
    pub components: Vec<ComponentDefRaw>,
    #[serde(default)]
    pub states: Option<StatesBlockRaw>,
}

#[derive(Debug, Clone)]
pub struct EntityDef {
    pub count: Option<ScalarExpr>,
    pub base_stats: HashMap<StatId, f32>,
    pub abilities: Vec<String>,
    pub components: Vec<ComponentDef>,
    pub states: Option<StatesBlock>,
}

impl StateDefRaw {
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

impl EntityDefRaw {
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
