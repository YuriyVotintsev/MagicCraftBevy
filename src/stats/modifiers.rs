use bevy::prelude::*;
use std::collections::HashSet;

use super::Stat;

#[derive(Debug, Clone)]
pub struct Modifier {
    pub stat: Stat,
    pub value: f32,
}

#[derive(Component, Default)]
pub struct Modifiers {
    list: Vec<Modifier>,
}

impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, stat: Stat, value: f32) {
        self.list.push(Modifier { stat, value });
    }

    pub fn sum(&self, stat: Stat) -> f32 {
        self.list
            .iter()
            .filter(|m| m.stat == stat)
            .map(|m| m.value)
            .sum()
    }

    pub fn product(&self, stat: Stat) -> f32 {
        let mut result = 1.0;
        for m in &self.list {
            if m.stat == stat {
                result *= 1.0 + m.value;
            }
        }
        result
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &Modifier> {
        self.list.iter()
    }

    pub fn affected_stats(&self) -> impl Iterator<Item = Stat> + '_ {
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
