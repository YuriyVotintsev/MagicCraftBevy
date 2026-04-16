use bevy::prelude::*;

use super::registry::{ModifierKind, Stat};
use super::{ComputedStats, DirtyStats, Modifiers};

#[derive(Resource)]
pub struct StatCalculators {
    calculation_order: Vec<Stat>,
    reverse_deps: [Vec<Stat>; Stat::COUNT],
}

impl StatCalculators {
    pub fn build() -> Self {
        let reverse_deps: [Vec<Stat>; Stat::COUNT] = std::array::from_fn(|_| Vec::new());
        let mut this = Self {
            calculation_order: Vec::new(),
            reverse_deps,
        };
        this.rebuild();
        this
    }

    fn rebuild(&mut self) {
        self.calculation_order = self.topological_sort();

        for deps in &mut self.reverse_deps {
            deps.clear();
        }

        for stat in Stat::iter() {
            for &dep in stat.deps() {
                self.reverse_deps[dep.index()].push(stat);
            }
        }
    }

    fn topological_sort(&self) -> Vec<Stat> {
        let n = Stat::COUNT;
        let mut in_degree: Vec<usize> = vec![0; n];
        let mut adjacency: Vec<Vec<Stat>> = vec![Vec::new(); n];

        for stat in Stat::iter() {
            for &dep in stat.deps() {
                adjacency[dep.index()].push(stat);
                in_degree[stat.index()] += 1;
            }
        }

        let mut queue: Vec<Stat> = Vec::new();
        for stat in Stat::iter() {
            if in_degree[stat.index()] == 0 {
                queue.push(stat);
            }
        }

        let mut result = Vec::new();
        while let Some(stat) = queue.pop() {
            result.push(stat);
            for &dependent in &adjacency[stat.index()] {
                in_degree[dependent.index()] -= 1;
                if in_degree[dependent.index()] == 0 {
                    queue.push(dependent);
                }
            }
        }

        if result.len() != n {
            let cycle_members: Vec<&'static str> = Stat::iter()
                .filter(|s| in_degree[s.index()] > 0)
                .map(|s| s.name())
                .collect();
            panic!(
                "Circular dependency detected in stats calculation! Stats involved: {:?}",
                cycle_members
            );
        }

        result
    }

    pub fn invalidate(&self, stat: Stat, dirty: &mut DirtyStats) {
        dirty.mark(stat);
        for &dependent in &self.reverse_deps[stat.index()] {
            if !dirty.stats.contains(&dependent) {
                self.invalidate(dependent, dirty);
            }
        }
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
                computed.set_bucket(stat, ModifierKind::Flat, modifiers.sum(stat, ModifierKind::Flat));
                computed.set_bucket(stat, ModifierKind::Increased, modifiers.sum(stat, ModifierKind::Increased));
                computed.set_bucket(stat, ModifierKind::More, modifiers.product(stat, ModifierKind::More));
                let final_value = computed.apply(stat, 0.0);
                computed.set_final(stat, final_value);
            }
        }

        dirty.clear();
    }
}
