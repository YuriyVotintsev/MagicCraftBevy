use std::collections::HashSet;

use crate::affixes::{AffixDefRaw, OrbDefRaw};
use crate::artifacts::ArtifactDefRaw;
use crate::player::hero_class::HeroClassRaw;
use crate::stats::modifier_def::{ModifierDefRaw, StatRangeRaw};

fn load_stat_names() -> HashSet<String> {
    #[derive(serde::Deserialize)]
    struct StatsConfig {
        stat_ids: Vec<crate::stats::loader::StatDefRaw>,
        #[allow(dead_code)]
        calculators: Vec<crate::stats::loader::CalculatorDefRaw>,
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
