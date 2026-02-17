use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use super::{StatId, StatRegistry};

#[derive(Clone, Deserialize)]
pub struct StatDisplayRuleRaw {
    pub stats: Vec<String>,
    pub format: String,
}

pub struct StatDisplayRule {
    pub stats: Vec<StatId>,
    pub format: String,
}

#[derive(Resource, Default)]
pub struct StatDisplayRegistry {
    rules: Vec<StatDisplayRule>,
    single_stat_index: HashMap<StatId, usize>,
}

impl StatDisplayRegistry {
    pub fn new(raw_rules: Vec<StatDisplayRuleRaw>, registry: &StatRegistry) -> Self {
        let mut rules = Vec::new();
        let mut single_stat_index = HashMap::new();

        for raw in raw_rules {
            let stats: Vec<StatId> = raw
                .stats
                .iter()
                .filter_map(|name| registry.get(name))
                .collect();
            if stats.len() != raw.stats.len() {
                continue;
            }
            let idx = rules.len();
            if stats.len() == 1 {
                single_stat_index.insert(stats[0], idx);
            }
            rules.push(StatDisplayRule {
                stats,
                format: raw.format,
            });
        }

        Self { rules, single_stat_index }
    }

    pub fn format(&self, stats: &[(StatId, f32)]) -> Vec<String> {
        if stats.len() > 1 {
            let stat_ids: Vec<StatId> = stats.iter().map(|(s, _)| *s).collect();
            for rule in &self.rules {
                if rule.stats == stat_ids {
                    return vec![self.apply_format(&rule.format, stats)];
                }
            }
        }

        stats
            .iter()
            .map(|(stat, value)| {
                if let Some(&idx) = self.single_stat_index.get(stat) {
                    self.apply_format(&self.rules[idx].format, &[(*stat, *value)])
                } else {
                    format_fallback(*value)
                }
            })
            .collect()
    }

    fn apply_format(&self, fmt: &str, stats: &[(StatId, f32)]) -> String {
        let is_percent = fmt.contains('%');
        let mut result = fmt.to_string();
        for (i, (_, value)) in stats.iter().enumerate() {
            let display = if is_percent {
                format!("{}", (*value * 100.0).round() as i32)
            } else {
                format!("{}", value.round() as i32)
            };
            result = result.replace(&format!("{{{}}}", i), &display);
        }
        result
    }
}

fn format_fallback(value: f32) -> String {
    if value > 0.0 {
        format!("+{}", value.round() as i32)
    } else {
        format!("{}", value.round() as i32)
    }
}
