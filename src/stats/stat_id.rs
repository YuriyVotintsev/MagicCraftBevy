use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct StatId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Product,
    Custom,
}

#[derive(Debug, Clone)]
pub struct StatDef {
    pub name: String,
    pub aggregation: AggregationType,
}

#[derive(Resource, Default)]
pub struct StatRegistry {
    name_to_id: HashMap<String, StatId>,
    stats: Vec<StatDef>,
}

impl StatRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: &str, aggregation: AggregationType) -> StatId {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }

        let id = StatId(self.stats.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.stats.push(StatDef {
            name: name.to_string(),
            aggregation,
        });
        id
    }

    pub fn get(&self, name: &str) -> Option<StatId> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_def(&self, id: StatId) -> Option<&StatDef> {
        self.stats.get(id.0 as usize)
    }

    pub fn name(&self, id: StatId) -> Option<&str> {
        self.stats.get(id.0 as usize).map(|d| d.name.as_str())
    }

    pub fn aggregation(&self, id: StatId) -> Option<AggregationType> {
        self.stats.get(id.0 as usize).map(|d| d.aggregation)
    }

    pub fn len(&self) -> usize {
        self.stats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (StatId, &StatDef)> {
        self.stats
            .iter()
            .enumerate()
            .map(|(i, def)| (StatId(i as u32), def))
    }
}
