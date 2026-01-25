use serde::Deserialize;
use std::fs;

use crate::expression::{Expression, ExpressionRaw};
use super::{AggregationType, StatCalculators, StatId, StatRegistry};

#[derive(Debug, Deserialize)]
pub struct StatDefRaw {
    pub name: String,
    pub aggregation: AggregationType,
}

#[derive(Debug, Deserialize)]
pub struct CalculatorDefRaw {
    pub stat: String,
    pub formula: ExpressionRaw,
    pub depends_on: Vec<String>,
}

pub fn load_stats(
    stat_ids_path: &str,
    calculators_path: &str,
) -> (StatRegistry, StatCalculators) {
    let stat_defs_content = fs::read_to_string(stat_ids_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read stat_ids file '{}': {}",
            stat_ids_path, e
        )
    });
    let stat_defs: Vec<StatDefRaw> = ron::from_str(&stat_defs_content).unwrap_or_else(|e| {
        panic!(
            "Failed to parse stat_ids RON '{}': {}\nContent:\n{}",
            stat_ids_path, e, stat_defs_content
        )
    });

    let mut registry = StatRegistry::new();
    for def in &stat_defs {
        registry.insert(&def.name, def.aggregation.clone());
    }

    let mut calculators = StatCalculators::new(registry.len());

    for def in &stat_defs {
        let stat_id = registry.get(&def.name).unwrap();
        match &def.aggregation {
            AggregationType::Sum => {
                calculators.set(stat_id, Expression::ModifierSum(stat_id), vec![]);
            }
            AggregationType::Product => {
                calculators.set(stat_id, Expression::ModifierProduct(stat_id), vec![]);
            }
            AggregationType::Standard { base, increased, more } => {
                let base_id = registry.get(base).unwrap_or_else(|| {
                    panic!(
                        "Standard aggregation for '{}' references unknown base stat: '{}'",
                        def.name, base
                    )
                });
                let increased_id = registry.get(increased).unwrap_or_else(|| {
                    panic!(
                        "Standard aggregation for '{}' references unknown increased stat: '{}'",
                        def.name, increased
                    )
                });
                let more_id = registry.get(more).unwrap_or_else(|| {
                    panic!(
                        "Standard aggregation for '{}' references unknown more stat: '{}'",
                        def.name, more
                    )
                });

                let formula = Expression::Mul(
                    Box::new(Expression::Mul(
                        Box::new(Expression::Stat(base_id)),
                        Box::new(Expression::Add(
                            Box::new(Expression::Constant(1.0)),
                            Box::new(Expression::Stat(increased_id)),
                        )),
                    )),
                    Box::new(Expression::Stat(more_id)),
                );

                let depends_on = vec![base_id, increased_id, more_id];
                calculators.set(stat_id, formula, depends_on);
            }
            AggregationType::Custom => {}
        }
    }

    let custom_calcs_content = fs::read_to_string(calculators_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read calculators file '{}': {}",
            calculators_path, e
        )
    });
    let custom_calcs: Vec<CalculatorDefRaw> = ron::from_str(&custom_calcs_content).unwrap_or_else(|e| {
        panic!(
            "Failed to parse calculators RON '{}': {}\nContent:\n{}",
            calculators_path, e, custom_calcs_content
        )
    });

    for calc in custom_calcs {
        let stat_id = registry.get(&calc.stat).unwrap_or_else(|| {
            panic!(
                "Calculator references unknown stat: '{}' (calculators file: {})",
                calc.stat, calculators_path
            )
        });
        let formula = calc.formula.resolve(&registry);
        let deps: Vec<StatId> = calc
            .depends_on
            .iter()
            .map(|s| {
                registry.get(s).unwrap_or_else(|| {
                    panic!(
                        "Calculator for '{}' references unknown dependency stat: '{}'",
                        calc.stat, s
                    )
                })
            })
            .collect();
        calculators.set(stat_id, formula, deps);
    }

    calculators.rebuild();

    (registry, calculators)
}
