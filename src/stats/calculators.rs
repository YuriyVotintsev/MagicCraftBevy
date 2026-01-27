use bevy::prelude::*;

use super::expression::Expression;
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
        let n = self.entries.len();
        let mut in_degree: Vec<usize> = vec![0; n];
        let mut adjacency: Vec<Vec<StatId>> = vec![Vec::new(); n];

        for (idx, entry) in self.entries.iter().enumerate() {
            if let Some(entry) = entry {
                for &dep in &entry.depends_on {
                    let dep_idx = dep.0 as usize;
                    if dep_idx < n {
                        adjacency[dep_idx].push(StatId(idx as u32));
                        in_degree[idx] += 1;
                    }
                }
            }
        }

        let mut queue: Vec<StatId> = Vec::new();
        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.is_some() && in_degree[idx] == 0 {
                queue.push(StatId(idx as u32));
            }
        }

        let mut result = Vec::new();
        while let Some(stat) = queue.pop() {
            result.push(stat);
            let idx = stat.0 as usize;
            for &dependent in &adjacency[idx] {
                let dep_idx = dependent.0 as usize;
                in_degree[dep_idx] -= 1;
                if in_degree[dep_idx] == 0 {
                    queue.push(dependent);
                }
            }
        }

        let expected_count = self.entries.iter().filter(|e| e.is_some()).count();
        if result.len() != expected_count {
            let cycle_members: Vec<u32> = in_degree
                .iter()
                .enumerate()
                .filter(|(idx, &deg)| deg > 0 && self.entries[*idx].is_some())
                .map(|(idx, _)| idx as u32)
                .collect();
            panic!(
                "Circular dependency detected in stats calculation! Stats involved: {:?}",
                cycle_members
            );
        }

        result
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn calculation_order(&self) -> &[StatId] {
        &self.calculation_order
    }

    pub fn recalculate(
        &self,
        modifiers: &Modifiers,
        computed: &mut ComputedStats,
        dirty: &mut DirtyStats,
    ) {
        if dirty.is_empty() {
            return;
        }

        for &stat in &self.calculation_order {
            if dirty.stats.contains(&stat) {
                let value = self.calculate(stat, modifiers, computed);
                computed.set(stat, value);
            }
        }

        dirty.clear();
    }
}
