use bevy::prelude::*;

use super::registry::{Stat, StatEvalKind};
use super::{ComputedStats, DirtyStats, Modifiers};

#[derive(Clone)]
struct CalculatorEntry {
    eval: StatEvalKind,
    depends_on: Vec<Stat>,
}

#[derive(Resource)]
pub struct StatCalculators {
    entries: [Option<CalculatorEntry>; Stat::COUNT],
    calculation_order: Vec<Stat>,
    reverse_deps: [Vec<Stat>; Stat::COUNT],
}

impl StatCalculators {
    pub fn build() -> Self {
        let entries: [Option<CalculatorEntry>; Stat::COUNT] = std::array::from_fn(|i| {
            let stat = Stat::ALL[i];
            let eval = stat.eval_kind();
            let depends_on = eval.dependencies();
            Some(CalculatorEntry { eval, depends_on })
        });

        let reverse_deps: [Vec<Stat>; Stat::COUNT] = std::array::from_fn(|_| Vec::new());

        let mut this = Self {
            entries,
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

        for (stat_idx, entry) in self.entries.iter().enumerate() {
            if let Some(entry) = entry {
                for &dep in &entry.depends_on {
                    self.reverse_deps[dep.index()].push(Stat::ALL[stat_idx]);
                }
            }
        }
    }

    fn topological_sort(&self) -> Vec<Stat> {
        let n = Stat::COUNT;
        let mut in_degree: Vec<usize> = vec![0; n];
        let mut adjacency: Vec<Vec<Stat>> = vec![Vec::new(); n];

        for (idx, entry) in self.entries.iter().enumerate() {
            if let Some(entry) = entry {
                for &dep in &entry.depends_on {
                    adjacency[dep.index()].push(Stat::ALL[idx]);
                    in_degree[idx] += 1;
                }
            }
        }

        let mut queue: Vec<Stat> = Vec::new();
        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.is_some() && in_degree[idx] == 0 {
                queue.push(Stat::ALL[idx]);
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

        let expected_count = self.entries.iter().filter(|e| e.is_some()).count();
        if result.len() != expected_count {
            let cycle_members: Vec<&'static str> = in_degree
                .iter()
                .enumerate()
                .filter(|(idx, &deg)| deg > 0 && self.entries[*idx].is_some())
                .map(|(idx, _)| Stat::ALL[idx].name())
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

    fn calculate(&self, stat: Stat, modifiers: &Modifiers, computed: &ComputedStats) -> f32 {
        let Some(entry) = &self.entries[stat.index()] else {
            return 0.0;
        };
        match &entry.eval {
            StatEvalKind::Sum => modifiers.sum(stat),
            StatEvalKind::Product => modifiers.product(stat),
            StatEvalKind::FlatIncreased { flat, increased } => {
                computed.get(*flat) * (1.0 + computed.get(*increased))
            }
            StatEvalKind::FlatIncreasedMore {
                flat,
                increased,
                more,
            } => computed.get(*flat) * (1.0 + computed.get(*increased)) * computed.get(*more),
            StatEvalKind::ClampedChance { flat, increased } => {
                (computed.get(*flat) * (1.0 + computed.get(*increased))).clamp(0.0, 1.0)
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
                let value = self.calculate(stat, modifiers, computed);
                computed.set(stat, value);
            }
        }

        dirty.clear();
    }
}
