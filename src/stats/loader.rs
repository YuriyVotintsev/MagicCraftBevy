use serde::Deserialize;
use std::fs;

use crate::expr::parser::{TypedExpr, StatAtomParser, parse_expr_string_with};
use super::stat_id::StatEvalKind;
use super::{StatCalculators, StatId, StatRegistry};

#[derive(Debug, Deserialize)]
pub struct StatDefRaw {
    pub name: String,
    pub eval: StatEvalKindRaw,
    #[serde(default)]
    pub lower_is_better: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub enum StatEvalKindRaw {
    Sum,
    Product,
    Formula(String),
}

fn resolve_formula(formula_str: &str, stat_name: &str, registry: &StatRegistry) -> (StatEvalKind, Vec<StatId>) {
    let parsed = match parse_expr_string_with(formula_str, &StatAtomParser) {
        Ok(TypedExpr::Scalar(expr)) => expr,
        Ok(_) => panic!("Stat '{}' formula must be a scalar expression, got vec/entity", stat_name),
        Err(e) => panic!("Failed to parse formula for stat '{}': {}\nFormula: {}", stat_name, e, formula_str),
    };

    let resolved = parsed.resolve(registry);

    let mut deps = Vec::new();
    resolved.collect_stat_deps(&mut deps);
    deps.dedup();

    (StatEvalKind::Formula(resolved), deps)
}

#[allow(dead_code)]
pub fn load_stats(config_path: &str) -> (StatRegistry, StatCalculators) {
    let content = fs::read_to_string(config_path).unwrap_or_else(|e| {
        panic!("Failed to read stats config '{}': {}", config_path, e)
    });

    #[derive(Deserialize)]
    struct StatsFile {
        stat_ids: Vec<StatDefRaw>,
    }

    let config: StatsFile = ron::from_str(&content).unwrap_or_else(|e| {
        panic!("Failed to parse stats config '{}': {}\nContent:\n{}", config_path, e, content)
    });

    let mut registry = StatRegistry::new();
    for def in &config.stat_ids {
        registry.insert(&def.name, def.lower_is_better);
    }

    let mut calculators = StatCalculators::new(registry.len());

    for def in &config.stat_ids {
        let stat_id = registry.get(&def.name).unwrap();
        match &def.eval {
            StatEvalKindRaw::Sum => {
                calculators.set(stat_id, StatEvalKind::Sum, vec![]);
            }
            StatEvalKindRaw::Product => {
                calculators.set(stat_id, StatEvalKind::Product, vec![]);
            }
            StatEvalKindRaw::Formula(formula_str) => {
                let (eval, deps) = resolve_formula(formula_str, &def.name, &registry);
                calculators.set(stat_id, eval, deps);
            }
        }
    }

    calculators.rebuild();

    (registry, calculators)
}
