use bevy::prelude::*;
use std::collections::HashSet;

use super::StatId;

#[derive(Debug, Clone)]
pub struct Modifier {
    pub stat: StatId,
    pub value: f32,
    #[allow(dead_code)]
    pub source: Option<Entity>,
}

#[derive(Component, Default)]
pub struct Modifiers {
    list: Vec<Modifier>,
}

impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, stat: StatId, value: f32, source: Option<Entity>) {
        self.list.push(Modifier { stat, value, source });
    }

    #[allow(dead_code)]
    pub fn remove_by_source(&mut self, source: Entity) -> Vec<StatId> {
        let mut affected = Vec::new();
        self.list.retain(|m| {
            if m.source == Some(source) {
                affected.push(m.stat);
                false
            } else {
                true
            }
        });
        affected
    }

    pub fn sum(&self, stat: StatId) -> f32 {
        self.list
            .iter()
            .filter(|m| m.stat == stat)
            .map(|m| m.value)
            .sum()
    }

    pub fn product(&self, stat: StatId) -> f32 {
        let mut result = 1.0;
        let mut found = false;
        for m in &self.list {
            if m.stat == stat {
                result *= m.value;
                found = true;
            }
        }
        if found { result } else { 1.0 }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &Modifier> {
        self.list.iter()
    }

    #[allow(dead_code)]
    pub fn affected_stats(&self) -> impl Iterator<Item = StatId> + '_ {
        let mut seen = HashSet::new();
        self.list.iter().filter_map(move |m| {
            if seen.insert(m.stat) {
                Some(m.stat)
            } else {
                None
            }
        })
    }
}
