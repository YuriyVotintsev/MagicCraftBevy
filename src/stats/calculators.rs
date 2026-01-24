use bevy::prelude::*;
use std::collections::HashMap;

use crate::expression::Expression;
use super::{ComputedStats, DirtyStats, Modifiers, StatId};

#[derive(Clone)]
pub struct CalculatorEntry {
    pub formula: Expression,
    pub depends_on: Vec<StatId>,
}

#[derive(Resource)]
pub struct StatCalculators {
    entries: Vec<Option<CalculatorEntry>>,
    calculation_order: Vec<StatId>,
    reverse_deps: Vec<Vec<StatId>>,
}

impl StatCalculators {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: vec![None; capacity],
            calculation_order: Vec::new(),
            reverse_deps: vec![Vec::new(); capacity],
        }
    }

    pub fn set(&mut self, stat: StatId, formula: Expression, depends_on: Vec<StatId>) {
        let idx = stat.0 as usize;
        if idx >= self.entries.len() {
            self.entries.resize_with(idx + 1, || None);
            self.reverse_deps.resize(idx + 1, Vec::new());
        }
        self.entries[idx] = Some(CalculatorEntry { formula, depends_on });
    }

    pub fn rebuild(&mut self) {
        self.calculation_order = self.topological_sort();

        for deps in &mut self.reverse_deps {
            deps.clear();
        }

        for (stat_idx, entry) in self.entries.iter().enumerate() {
            if let Some(entry) = entry {
                for &dep in &entry.depends_on {
                    let dep_idx = dep.0 as usize;
                    if dep_idx < self.reverse_deps.len() {
                        self.reverse_deps[dep_idx].push(StatId(stat_idx as u32));
                    }
                }
            }
        }
    }

    fn topological_sort(&self) -> Vec<StatId> {
        let mut result = Vec::new();
        let mut visited = HashMap::new();

        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.is_some() {
                self.visit(StatId(idx as u32), &mut visited, &mut result);
            }
        }

        result
    }

    fn visit(&self, stat: StatId, visited: &mut HashMap<StatId, bool>, result: &mut Vec<StatId>) {
        if let Some(&in_progress) = visited.get(&stat) {
            if in_progress {
                panic!("Circular dependency detected for stat {:?}", stat.0);
            }
            return;
        }

        visited.insert(stat, true);

        if let Some(Some(entry)) = self.entries.get(stat.0 as usize) {
            for &dep in &entry.depends_on {
                self.visit(dep, visited, result);
            }
        }

        visited.insert(stat, false);
        result.push(stat);
    }

    pub fn invalidate(&self, stat: StatId, dirty: &mut DirtyStats) {
        dirty.mark(stat);
        let idx = stat.0 as usize;
        if idx < self.reverse_deps.len() {
            for &dependent in &self.reverse_deps[idx] {
                if !dirty.stats.contains(&dependent) {
                    self.invalidate(dependent, dirty);
                }
            }
        }
    }

    pub fn calculate(&self, stat: StatId, modifiers: &Modifiers, computed: &ComputedStats) -> f32 {
        if let Some(Some(entry)) = self.entries.get(stat.0 as usize) {
            entry.formula.evaluate(modifiers, computed)
        } else {
            0.0
        }
    }

    pub fn calculation_order(&self) -> &[StatId] {
        &self.calculation_order
    }
}
