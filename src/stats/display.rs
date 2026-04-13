#![expect(dead_code, reason = "stat display registry — scaffolding for shop perk UI, not wired yet")]

use std::collections::HashMap;

use bevy::prelude::*;

use super::Stat;

#[derive(Clone, Debug)]
pub enum SignMode {
    Default,
    ShowSign,
    Absolute,
}

#[derive(Clone, Debug)]
pub struct ValueTemplate {
    pub prefix: String,
    pub suffix: String,
    pub sign_mode: SignMode,
}

#[derive(Clone, Debug)]
pub enum FormatSpan {
    Value {
        index: usize,
        is_percent: bool,
        lower_is_better: bool,
        template: ValueTemplate,
        negative_template: Option<ValueTemplate>,
    },
    Label(String),
}

struct MultiStatEntry {
    stats: Vec<Stat>,
    lines: Vec<Vec<FormatSpan>>,
}

#[derive(Resource, Default)]
pub struct StatDisplayRegistry {
    single_stat_formats: HashMap<Stat, Vec<FormatSpan>>,
    multi_stat_formats: HashMap<Vec<Stat>, MultiStatEntry>,
}

const DISPLAY_RULES: &[(&[Stat], &str)] = &[
    (&[Stat::PhysicalDamageFlat], "[{+0}] physical damage"),
    (&[Stat::PhysicalDamageIncreased], "[{+0}%] physical damage"),
    (&[Stat::PhysicalDamageMore], "[{|0|}% more;{|0|}% less] physical damage"),
    (&[Stat::MaxLifeFlat], "[{+0}] max life"),
    (&[Stat::MaxLifeIncreased], "[{+0}%] max life"),
    (&[Stat::MaxLifeMore], "[{|0|}% more;{|0|}% less] max life"),
    (&[Stat::MovementSpeedFlat], "[{+0}] movement speed"),
    (&[Stat::MovementSpeedIncreased], "[{+0}%] movement speed"),
    (&[Stat::CritChanceFlat], "[{+0}%] crit chance"),
    (&[Stat::CritMultiplier], "[{+0}%] crit multiplier"),
    (&[Stat::ProjectileSpeedFlat], "[{+0}] projectile speed"),
    (&[Stat::ProjectileSpeedIncreased], "[{+0}%] projectile speed"),
    (&[Stat::ProjectileCount], "[{+0}] projectile"),
    (&[Stat::MaxManaFlat], "[{+0}] max mana"),
    (&[Stat::MaxManaIncreased], "[{+0}%] max mana"),
    (&[Stat::AreaOfEffectFlat], "[{+0}] area of effect"),
    (&[Stat::AreaOfEffectIncreased], "[{+0}%] area of effect"),
    (&[Stat::DurationFlat], "[{+0}] duration"),
    (&[Stat::DurationIncreased], "[{+0}%] duration"),
    (&[Stat::PickupRadiusFlat], "[{+0}] pickup radius"),
    (&[Stat::PickupRadiusIncreased], "[{+0}%] pickup radius"),
];

fn parse_template_half(half: &str) -> Option<(usize, ValueTemplate)> {
    let brace_start = half.find('{')?;
    let rel_brace_end = half[brace_start + 1..].find('}')?;
    let brace_end = brace_start + 1 + rel_brace_end;
    let inside = &half[brace_start + 1..brace_end];

    let (sign_mode, index_str) = if inside.starts_with('+') {
        (SignMode::ShowSign, &inside[1..])
    } else if inside.starts_with('|') && inside.ends_with('|') && inside.len() >= 3 {
        (SignMode::Absolute, &inside[1..inside.len() - 1])
    } else {
        (SignMode::Default, inside)
    };

    let index = index_str.parse::<usize>().ok()?;
    let prefix = half[..brace_start].to_string();
    let suffix = half[brace_end + 1..].to_string();

    Some((index, ValueTemplate { prefix, suffix, sign_mode }))
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

                let (pos_half, neg_half) = if let Some(semi) = inside.find(';') {
                    (&inside[..semi], Some(&inside[semi + 1..]))
                } else {
                    (inside, None)
                };

                if let Some((index, template)) = parse_template_half(pos_half) {
                    let negative_template = neg_half.and_then(|h| {
                        parse_template_half(h).map(|(_, t)| t)
                    });

                    let is_percent = template.suffix.contains('%')
                        || negative_template.as_ref().is_some_and(|t| t.suffix.contains('%'));

                    spans.push(FormatSpan::Value {
                        index,
                        is_percent,
                        lower_is_better: false,
                        template,
                        negative_template,
                    });
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
    pub fn build() -> Self {
        let mut single_stat_formats: HashMap<Stat, Vec<FormatSpan>> = HashMap::new();
        let mut multi_stat_formats: HashMap<Vec<Stat>, MultiStatEntry> = HashMap::new();

        for (stats, format) in DISPLAY_RULES {
            let spans = parse_format_string(format);
            if stats.len() == 1 {
                single_stat_formats.insert(stats[0], spans);
            } else {
                let mut sorted_key = stats.to_vec();
                sorted_key.sort_by_key(|s| s.index());
                multi_stat_formats.insert(
                    sorted_key,
                    MultiStatEntry {
                        stats: stats.to_vec(),
                        lines: vec![spans],
                    },
                );
            }
        }

        for &stat in Stat::ALL {
            if !single_stat_formats.contains_key(&stat) {
                let title = to_title_case(stat.name());
                let fallback_fmt = format!("[{{+0}}] {}", title);
                single_stat_formats.insert(stat, parse_format_string(&fallback_fmt));
            }
        }

        Self {
            single_stat_formats,
            multi_stat_formats,
        }
    }

    pub fn get_format(&self, stats: &[Stat]) -> Vec<Vec<FormatSpan>> {
        if stats.len() == 1 {
            if let Some(spans) = self.single_stat_formats.get(&stats[0]) {
                return vec![spans.clone()];
            }
            return vec![vec![FormatSpan::Value {
                index: 0,
                is_percent: false,
                lower_is_better: false,
                template: ValueTemplate {
                    prefix: String::new(),
                    suffix: String::new(),
                    sign_mode: SignMode::ShowSign,
                },
                negative_template: None,
            }]];
        }

        let mut sorted_key: Vec<Stat> = stats.to_vec();
        sorted_key.sort_by_key(|s| s.index());

        if let Some(entry) = self.multi_stat_formats.get(&sorted_key) {
            let mut remap = vec![0usize; entry.stats.len()];
            for (rule_pos, stat) in entry.stats.iter().enumerate() {
                if let Some(caller_pos) = stats.iter().position(|s| s == stat) {
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
                                is_percent,
                                lower_is_better,
                                template,
                                negative_template,
                            } => FormatSpan::Value {
                                index: remap.get(*index).copied().unwrap_or(*index),
                                is_percent: *is_percent,
                                lower_is_better: *lower_is_better,
                                template: template.clone(),
                                negative_template: negative_template.clone(),
                            },
                            other => other.clone(),
                        })
                        .collect()
                })
                .collect();
        }

        stats
            .iter()
            .filter_map(|stat| self.single_stat_formats.get(stat).cloned())
            .collect()
    }
}
