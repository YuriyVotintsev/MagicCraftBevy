use bevy::prelude::*;
use std::collections::HashMap;

pub use crate::expr::StatId;

use crate::expr::ScalarExpr;

#[derive(Debug, Clone)]
pub enum StatEvalKind {
    Sum,
    Product,
    Formula(ScalarExpr),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StatDef {
    pub name: String,
    pub lower_is_better: bool,
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

    pub fn insert(&mut self, name: &str, lower_is_better: bool) -> StatId {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }

        let id = StatId(self.stats.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.stats.push(StatDef {
            name: name.to_string(),
            lower_is_better,
        });
        id
    }

    pub fn get(&self, name: &str) -> Option<StatId> {
        self.name_to_id.get(name).copied()
    }

    #[allow(dead_code)]
    pub fn get_def(&self, id: StatId) -> Option<&StatDef> {
        self.stats.get(id.0 as usize)
    }

    #[allow(dead_code)]
    pub fn name(&self, id: StatId) -> Option<&str> {
        self.stats.get(id.0 as usize).map(|d| d.name.as_str())
    }

    pub fn len(&self) -> usize {
        self.stats.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (StatId, &StatDef)> {
        self.stats
            .iter()
            .enumerate()
            .map(|(i, def)| (StatId(i as u32), def))
    }
}
