use bevy::prelude::*;
use std::collections::HashMap;

use super::{ComputedStats, DirtyStats, RawStats, StatId};

pub type CalculatorFn = fn(&RawStats, &ComputedStats) -> f32;

pub struct StatCalculatorEntry {
    pub calculate: CalculatorFn,
    pub depends_on: Vec<StatId>,
    pub invalidated_by: Vec<StatId>,
}

#[derive(Resource)]
pub struct StatCalculators {
    calculators: HashMap<StatId, StatCalculatorEntry>,
    calculation_order: Vec<StatId>,
    invalidation_map: HashMap<StatId, Vec<StatId>>,
}

impl Default for StatCalculators {
    fn default() -> Self {
        let mut calc = Self {
            calculators: HashMap::new(),
            calculation_order: Vec::new(),
            invalidation_map: HashMap::new(),
        };
        calc.register_all();
        calc.rebuild();
        calc
    }
}

impl StatCalculators {
    fn register(
        &mut self,
        stat: StatId,
        calculate: CalculatorFn,
        depends_on: Vec<StatId>,
        invalidated_by: Vec<StatId>,
    ) {
        self.calculators.insert(
            stat,
            StatCalculatorEntry {
                calculate,
                depends_on,
                invalidated_by,
            },
        );
    }

    fn register_all(&mut self) {
        self.register(
            StatId::Strength,
            |raw, _computed| {
                let base = raw.get(StatId::Strength);
                let increased = raw.get(StatId::StrengthIncreased);
                base * (1.0 + increased)
            },
            vec![],
            vec![StatId::Strength, StatId::StrengthIncreased],
        );

        self.register(
            StatId::MaxLife,
            |raw, computed| {
                let base = raw.get(StatId::MaxLife);
                let increased = raw.get(StatId::MaxLifeIncreased);
                let more = raw.get(StatId::MaxLifeMore);
                let per_str = raw.get(StatId::MaxLifePerStrength);
                let strength = computed.get(StatId::Strength);

                let more_mult = if more == 0.0 { 1.0 } else { more };
                (base + strength * per_str) * (1.0 + increased) * more_mult
            },
            vec![StatId::Strength],
            vec![
                StatId::MaxLife,
                StatId::MaxLifeIncreased,
                StatId::MaxLifeMore,
                StatId::MaxLifePerStrength,
            ],
        );

        self.register(
            StatId::MaxMana,
            |raw, _computed| {
                let base = raw.get(StatId::MaxMana);
                let increased = raw.get(StatId::MaxManaIncreased);
                base * (1.0 + increased)
            },
            vec![],
            vec![StatId::MaxMana, StatId::MaxManaIncreased],
        );

        self.register(
            StatId::PhysicalDamage,
            |raw, _computed| {
                let base = raw.get(StatId::PhysicalDamage);
                let increased = raw.get(StatId::PhysicalDamageIncreased);
                let more = raw.get(StatId::PhysicalDamageMore);

                let more_mult = if more == 0.0 { 1.0 } else { more };
                base * (1.0 + increased) * more_mult
            },
            vec![],
            vec![
                StatId::PhysicalDamage,
                StatId::PhysicalDamageIncreased,
                StatId::PhysicalDamageMore,
            ],
        );

        self.register(
            StatId::MovementSpeed,
            |raw, _computed| {
                let base = raw.get(StatId::MovementSpeed);
                let increased = raw.get(StatId::MovementSpeedIncreased);
                let more = raw.get(StatId::MovementSpeedMore);

                let more_mult = if more == 0.0 { 1.0 } else { more };
                base * (1.0 + increased) * more_mult
            },
            vec![],
            vec![
                StatId::MovementSpeed,
                StatId::MovementSpeedIncreased,
                StatId::MovementSpeedMore,
            ],
        );

        self.register(
            StatId::ProjectileSpeed,
            |raw, _computed| {
                let base = raw.get(StatId::ProjectileSpeed);
                let increased = raw.get(StatId::ProjectileSpeedIncreased);
                base * (1.0 + increased)
            },
            vec![],
            vec![StatId::ProjectileSpeed, StatId::ProjectileSpeedIncreased],
        );

        self.register(
            StatId::ProjectileCount,
            |raw, _computed| raw.get(StatId::ProjectileCount).max(1.0),
            vec![],
            vec![StatId::ProjectileCount],
        );

        self.register(
            StatId::CritChance,
            |raw, _computed| {
                let base = raw.get(StatId::CritChance);
                let increased = raw.get(StatId::CritChanceIncreased);
                (base * (1.0 + increased)).clamp(0.0, 1.0)
            },
            vec![],
            vec![StatId::CritChance, StatId::CritChanceIncreased],
        );

        self.register(
            StatId::CritMultiplier,
            |raw, _computed| {
                let base = raw.get(StatId::CritMultiplier);
                let increased = raw.get(StatId::CritMultiplierIncreased);
                (base * (1.0 + increased)).max(1.0)
            },
            vec![],
            vec![StatId::CritMultiplier, StatId::CritMultiplierIncreased],
        );
    }

    fn rebuild(&mut self) {
        self.calculation_order = self.topological_sort();

        self.invalidation_map.clear();
        for (&computed_stat, entry) in &self.calculators {
            for &raw_stat in &entry.invalidated_by {
                self.invalidation_map
                    .entry(raw_stat)
                    .or_default()
                    .push(computed_stat);
            }
        }
    }

    fn topological_sort(&self) -> Vec<StatId> {
        let mut result = Vec::new();
        let mut visited = HashMap::new();

        for &stat in self.calculators.keys() {
            self.visit(stat, &mut visited, &mut result);
        }

        result
    }

    fn visit(
        &self,
        stat: StatId,
        visited: &mut HashMap<StatId, bool>,
        result: &mut Vec<StatId>,
    ) {
        if let Some(&in_progress) = visited.get(&stat) {
            if in_progress {
                panic!("Circular dependency detected for {:?}", stat);
            }
            return;
        }

        visited.insert(stat, true);

        if let Some(entry) = self.calculators.get(&stat) {
            for &dep in &entry.depends_on {
                self.visit(dep, visited, result);
            }
        }

        visited.insert(stat, false);
        result.push(stat);
    }

    pub fn invalidate(&self, raw_stat: StatId, dirty: &mut DirtyStats) {
        if let Some(computed_stats) = self.invalidation_map.get(&raw_stat) {
            for &computed_stat in computed_stats {
                dirty.mark(computed_stat);
                self.invalidate_dependents(computed_stat, dirty);
            }
        }
    }

    fn invalidate_dependents(&self, stat: StatId, dirty: &mut DirtyStats) {
        for (&other_stat, entry) in &self.calculators {
            if entry.depends_on.contains(&stat) && dirty.stats.insert(other_stat) {
                self.invalidate_dependents(other_stat, dirty);
            }
        }
    }

    pub fn calculate(&self, stat: StatId, raw: &RawStats, computed: &ComputedStats) -> f32 {
        if let Some(entry) = self.calculators.get(&stat) {
            (entry.calculate)(raw, computed)
        } else {
            0.0
        }
    }

    pub fn calculation_order(&self) -> &[StatId] {
        &self.calculation_order
    }
}
