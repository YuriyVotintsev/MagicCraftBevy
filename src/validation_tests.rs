use std::collections::HashSet;

use crate::affixes::{AffixDefRaw, OrbDefRaw};
use crate::artifacts::ArtifactDefRaw;
use crate::player::hero_class::HeroClassRaw;
use crate::stats::modifier_def::{ModifierDefRaw, StatRangeRaw};

fn load_stat_names() -> HashSet<String> {
    #[derive(serde::Deserialize)]
    struct StatsConfig {
        stat_ids: Vec<crate::stats::loader::StatDefRaw>,
        #[serde(default)]
        #[allow(dead_code)]
        calcs: Vec<crate::expr::calc::CalcTemplateRaw>,
        #[serde(default)]
        #[allow(dead_code)]
        display: Vec<crate::stats::display::StatDisplayRuleRaw>,
    }
    let content = std::fs::read_to_string("assets/stats/config.stats.ron").unwrap();
    let config: StatsConfig = ron::from_str(&content).unwrap();
    config.stat_ids.iter().map(|s| s.name.clone()).collect()
}

fn validate_modifier_stats(
    context: &str,
    modifiers: &[ModifierDefRaw],
    stat_names: &HashSet<String>,
    errors: &mut Vec<String>,
) {
    for modifier in modifiers {
        for sr in &modifier.stats {
            let stat = match sr {
                StatRangeRaw::Fixed { stat, .. } => stat,
                StatRangeRaw::Range { stat, .. } => stat,
            };
            if !stat_names.contains(stat) {
                errors.push(format!("{}: unknown stat '{}'", context, stat));
            }
        }
    }
}

fn find_ron_files(dir: &std::path::Path, suffix: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    find_ron_files_recursive(dir, suffix, &mut files);
    files
}

fn find_ron_files_recursive(dir: &std::path::Path, suffix: &str, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            find_ron_files_recursive(&path, suffix, out);
        } else if path.file_name().unwrap().to_string_lossy().ends_with(suffix) {
            out.push(path);
        }
    }
}

#[test]
fn validate_artifact_stats() {
    let stat_names = load_stat_names();
    let mut errors = Vec::new();

    for path in find_ron_files(std::path::Path::new("assets/artifacts"), ".artifact.ron") {
        let content = std::fs::read_to_string(&path).unwrap();
        let raw: ArtifactDefRaw = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
        validate_modifier_stats(
            &format!("Artifact '{}'", raw.id),
            &raw.modifiers,
            &stat_names,
            &mut errors,
        );
    }

    if !errors.is_empty() {
        panic!("Artifact validation errors:\n{}", errors.join("\n"));
    }
}

#[test]
fn validate_affix_stats() {
    let stat_names = load_stat_names();
    let mut errors = Vec::new();

    for path in find_ron_files(std::path::Path::new("assets/affixes"), ".affixes.ron") {
        let content = std::fs::read_to_string(&path).unwrap();
        let pool: Vec<AffixDefRaw> = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
        for raw in &pool {
            for (tier_idx, tier) in raw.tiers.iter().enumerate() {
                validate_modifier_stats(
                    &format!("Affix '{}' tier {}", raw.id, tier_idx),
                    std::slice::from_ref(tier),
                    &stat_names,
                    &mut errors,
                );
            }
        }
    }

    if !errors.is_empty() {
        panic!("Affix validation errors:\n{}", errors.join("\n"));
    }
}

#[test]
fn validate_hero_class_stats() {
    let stat_names = load_stat_names();
    let mut errors = Vec::new();

    for path in find_ron_files(std::path::Path::new("assets/heroes"), ".class.ron") {
        let content = std::fs::read_to_string(&path).unwrap();
        let raw: HeroClassRaw = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
        validate_modifier_stats(
            &format!("Hero class '{}'", raw.id),
            &raw.modifiers,
            &stat_names,
            &mut errors,
        );
    }

    if !errors.is_empty() {
        panic!("Hero class validation errors:\n{}", errors.join("\n"));
    }
}

#[test]
fn validate_orb_config() {
    let path = std::path::Path::new("assets/orbs/config.orbs.ron");
    let content = std::fs::read_to_string(path).unwrap();
    let orbs: Vec<OrbDefRaw> = ron::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    assert!(!orbs.is_empty(), "Orb config is empty");
}

#[test]
fn validate_stat_formulas() {
    use crate::expr::calc::{CalcRegistry, CalcTemplateRaw};
    use crate::expr::parser::{TypedExpr, StatAtomParser, parse_expr_string_with};
    use crate::stats::loader::{StatDefRaw, StatEvalKindRaw};
    use crate::stats::StatRegistry;

    #[derive(serde::Deserialize)]
    struct StatsConfig {
        stat_ids: Vec<StatDefRaw>,
        #[serde(default)]
        calcs: Vec<CalcTemplateRaw>,
    }

    let content = std::fs::read_to_string("assets/stats/config.stats.ron").unwrap();
    let config: StatsConfig = ron::from_str(&content).unwrap();
    let calc_registry = CalcRegistry::from_raw(&config.calcs);

    let mut registry = StatRegistry::new();
    for def in &config.stat_ids {
        registry.insert(&def.name, def.lower_is_better);
    }

    let mut errors = Vec::new();

    for def in &config.stat_ids {
        if let StatEvalKindRaw::Formula(formula_str) = &def.eval {
            let expanded = match calc_registry.expand(formula_str) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(format!("Stat '{}': calc expansion failed: {}", def.name, e));
                    continue;
                }
            };

            match parse_expr_string_with(&expanded, &StatAtomParser) {
                Ok(TypedExpr::Scalar(raw)) => {
                    let resolved = raw.resolve(&|name: &str| registry.get(name));
                    let mut deps = Vec::new();
                    resolved.collect_stat_deps(&mut deps);
                    if let Some(self_id) = registry.get(&def.name) {
                        if deps.contains(&self_id) {
                            errors.push(format!("Stat '{}': formula references itself", def.name));
                        }
                    }
                }
                Ok(_) => {
                    errors.push(format!("Stat '{}': formula must be a scalar expression", def.name));
                }
                Err(e) => {
                    errors.push(format!("Stat '{}': parse error: {}\n  Expanded: {}", def.name, e, expanded));
                }
            }
        }
    }

    if !errors.is_empty() {
        panic!("Stat formula validation errors:\n{}", errors.join("\n"));
    }
}

#[test]
fn validate_calc_templates() {
    use crate::expr::calc::{CalcRegistry, CalcTemplateRaw};

    #[derive(serde::Deserialize)]
    struct StatsConfig {
        #[serde(default)]
        #[allow(dead_code)]
        stat_ids: Vec<crate::stats::loader::StatDefRaw>,
        #[serde(default)]
        calcs: Vec<CalcTemplateRaw>,
    }

    let content = std::fs::read_to_string("assets/stats/config.stats.ron").unwrap();
    let config: StatsConfig = ron::from_str(&content).unwrap();
    assert!(!config.calcs.is_empty(), "No calc templates found");

    let calc_registry = CalcRegistry::from_raw(&config.calcs);

    let mut errors = Vec::new();
    for raw in &config.calcs {
        let test_args: Vec<String> = raw.params.iter()
            .map(|p| format!("test_{}", p))
            .collect();
        let call = format!("calc({}, {})", raw.name, test_args.join(", "));
        if let Err(e) = calc_registry.expand(&call) {
            errors.push(format!("Template '{}': expansion failed: {}", raw.name, e));
        }
    }

    if !errors.is_empty() {
        panic!("Calc template validation errors:\n{}", errors.join("\n"));
    }
}
