use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use super::{StatId, StatRegistry};

#[derive(Clone, Debug)]
pub enum FormatSpan {
    Value {
        index: usize,
        prefix: String,
        suffix: String,
        is_percent: bool,
    },
    Label(String),
}

#[derive(Clone, Deserialize)]
pub struct StatDisplayRuleRaw {
    pub stats: Vec<String>,
    pub format: String,
}

struct MultiStatEntry {
    stats: Vec<StatId>,
    lines: Vec<Vec<FormatSpan>>,
}

#[derive(Resource, Default)]
pub struct StatDisplayRegistry {
    single_stat_formats: HashMap<StatId, Vec<FormatSpan>>,
    multi_stat_formats: HashMap<Vec<StatId>, MultiStatEntry>,
}

fn parse_format_string(fmt: &str) -> Vec<FormatSpan> {
    let mut spans = Vec::new();
    let mut rest = fmt;

    while !rest.is_empty() {
        if let Some(bracket_start) = rest.find('[') {
            if bracket_start > 0 {
                spans.push(FormatSpan::Label(rest[..bracket_start].to_string()));
            }

            if let Some(rel_end) = rest[bracket_start + 1..].find(']') {
                let bracket_end = bracket_start + 1 + rel_end;
                let inside = &rest[bracket_start + 1..bracket_end];

                if let Some(brace_start) = inside.find('{') {
                    if let Some(rel_brace_end) = inside[brace_start + 1..].find('}') {
                        let brace_end = brace_start + 1 + rel_brace_end;
                        let index_str = &inside[brace_start + 1..brace_end];
                        if let Ok(index) = index_str.parse::<usize>() {
                            let prefix = inside[..brace_start].to_string();
                            let suffix = inside[brace_end + 1..].to_string();
                            let is_percent = suffix.contains('%');
                            spans.push(FormatSpan::Value {
                                index,
                                prefix,
                                suffix,
                                is_percent,
                            });
                        }
                    }
                }

                rest = &rest[bracket_end + 1..];
            } else {
                spans.push(FormatSpan::Label(rest.to_string()));
                break;
            }
        } else {
            spans.push(FormatSpan::Label(rest.to_string()));
            break;
        }
    }

    spans
}

fn to_title_case(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl StatDisplayRegistry {
    pub fn new(raw_rules: Vec<StatDisplayRuleRaw>, registry: &StatRegistry) -> Self {
        let mut single_stat_formats: HashMap<StatId, Vec<FormatSpan>> = HashMap::new();
        let mut multi_stat_formats: HashMap<Vec<StatId>, MultiStatEntry> = HashMap::new();

        for raw in &raw_rules {
            let stats: Vec<StatId> = raw
                .stats
                .iter()
                .filter_map(|name| registry.get(name))
                .collect();
            if stats.len() != raw.stats.len() {
                continue;
            }

            let spans = parse_format_string(&raw.format);

            if stats.len() == 1 {
                single_stat_formats.insert(stats[0], spans);
            } else {
                let mut sorted_key = stats.clone();
                sorted_key.sort_by_key(|s| s.0);
                multi_stat_formats.insert(
                    sorted_key,
                    MultiStatEntry {
                        stats,
                        lines: vec![spans],
                    },
                );
            }
        }

        for (stat_id, def) in registry.iter() {
            if !single_stat_formats.contains_key(&stat_id) {
                let title = to_title_case(&def.name);
                let fallback_fmt = format!("[+{{0}}] {}", title);
                single_stat_formats.insert(stat_id, parse_format_string(&fallback_fmt));
            }
        }

        Self {
            single_stat_formats,
            multi_stat_formats,
        }
    }

    pub fn get_format(&self, stat_ids: &[StatId]) -> Vec<Vec<FormatSpan>> {
        if stat_ids.len() == 1 {
            if let Some(spans) = self.single_stat_formats.get(&stat_ids[0]) {
                return vec![spans.clone()];
            }
            return vec![vec![FormatSpan::Value {
                index: 0,
                prefix: "+".to_string(),
                suffix: String::new(),
                is_percent: false,
            }]];
        }

        let mut sorted_key: Vec<StatId> = stat_ids.to_vec();
        sorted_key.sort_by_key(|s| s.0);

        if let Some(entry) = self.multi_stat_formats.get(&sorted_key) {
            let mut remap = vec![0usize; entry.stats.len()];
            for (rule_pos, stat) in entry.stats.iter().enumerate() {
                if let Some(caller_pos) = stat_ids.iter().position(|s| s == stat) {
                    remap[rule_pos] = caller_pos;
                }
            }

            return entry
                .lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|span| match span {
                            FormatSpan::Value {
                                index,
                                prefix,
                                suffix,
                                is_percent,
                            } => FormatSpan::Value {
                                index: remap.get(*index).copied().unwrap_or(*index),
                                prefix: prefix.clone(),
                                suffix: suffix.clone(),
                                is_percent: *is_percent,
                            },
                            other => other.clone(),
                        })
                        .collect()
                })
                .collect();
        }

        stat_ids
            .iter()
            .filter_map(|stat| self.single_stat_formats.get(stat).cloned())
            .collect()
    }
}
