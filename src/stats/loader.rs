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
    let stat_defs_content = fs::read_to_string(stat_ids_path)
        .expect(&format!("Failed to read stat_ids file: {}", stat_ids_path));
    let stat_defs: Vec<StatDefRaw> = ron::from_str(&stat_defs_content)
        .expect(&format!("Failed to parse stat_ids RON: {}", stat_ids_path));

    let mut registry = StatRegistry::new();
    for def in &stat_defs {
        registry.insert(&def.name, def.aggregation);
    }

    let mut calculators = StatCalculators::new(registry.len());

    for def in &stat_defs {
        let stat_id = registry.get(&def.name).unwrap();
        match def.aggregation {
            AggregationType::Sum => {
                calculators.set(stat_id, Expression::ModifierSum(stat_id), vec![]);
            }
            AggregationType::Product => {
                calculators.set(stat_id, Expression::ModifierProduct(stat_id), vec![]);
            }
            AggregationType::Custom => {}
        }
    }

    let custom_calcs_content = fs::read_to_string(calculators_path)
        .expect(&format!("Failed to read calculators file: {}", calculators_path));
    let custom_calcs: Vec<CalculatorDefRaw> = ron::from_str(&custom_calcs_content)
        .expect(&format!("Failed to parse calculators RON: {}", calculators_path));

    for calc in custom_calcs {
        let stat_id = registry
            .get(&calc.stat)
            .expect(&format!("Unknown stat in calculator: {}", calc.stat));
        let formula = calc.formula.resolve(&registry);
        let deps: Vec<StatId> = calc
            .depends_on
            .iter()
            .map(|s| registry.get(s).expect(&format!("Unknown dependency stat: {}", s)))
            .collect();
        calculators.set(stat_id, formula, deps);
    }

    calculators.rebuild();

    (registry, calculators)
}
