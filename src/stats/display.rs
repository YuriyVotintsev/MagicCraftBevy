use std::collections::HashMap;

use bevy::prelude::*;

use super::registry::{ModifierKind, Stat};

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

type StatKey = (Stat, ModifierKind);

struct MultiStatEntry {
    stats: Vec<StatKey>,
    lines: Vec<Vec<FormatSpan>>,
}

#[derive(Resource, Default)]
pub struct StatDisplayRegistry {
    single_stat_formats: HashMap<StatKey, Vec<FormatSpan>>,
    multi_stat_formats: HashMap<Vec<StatKey>, MultiStatEntry>,
    snapshot_formats: HashMap<Stat, Vec<FormatSpan>>,
}

const DISPLAY_RULES: &[(&[StatKey], &str)] = &[
    (&[(Stat::PhysicalDamage, ModifierKind::Flat)], "[{+0}] physical damage"),
    (&[(Stat::PhysicalDamage, ModifierKind::Increased)], "[{+0}%] physical damage"),
    (&[(Stat::PhysicalDamage, ModifierKind::More)], "[{|0|}% more;{|0|}% less] physical damage"),
    (&[(Stat::MaxLife, ModifierKind::Flat)], "[{+0}] max life"),
    (&[(Stat::MaxLife, ModifierKind::Increased)], "[{+0}%] max life"),
    (&[(Stat::MaxLife, ModifierKind::More)], "[{|0|}% more;{|0|}% less] max life"),
    (&[(Stat::MovementSpeed, ModifierKind::Flat)], "[{+0}] movement speed"),
    (&[(Stat::MovementSpeed, ModifierKind::Increased)], "[{+0}%] movement speed"),
    (&[(Stat::CritChance, ModifierKind::Flat)], "[{+0}%] crit chance"),
    (&[(Stat::CritMultiplier, ModifierKind::Flat)], "[{+0}%] crit multiplier"),
    (&[(Stat::ProjectileSpeed, ModifierKind::Flat)], "[{+0}] projectile speed"),
    (&[(Stat::ProjectileSpeed, ModifierKind::Increased)], "[{+0}%] projectile speed"),
    (&[(Stat::ProjectileCount, ModifierKind::Flat)], "[{+0}] projectile"),
    (&[(Stat::MaxMana, ModifierKind::Flat)], "[{+0}] max mana"),
    (&[(Stat::MaxMana, ModifierKind::Increased)], "[{+0}%] max mana"),
    (&[(Stat::AreaOfEffect, ModifierKind::Flat)], "[{+0}] area of effect"),
    (&[(Stat::AreaOfEffect, ModifierKind::Increased)], "[{+0}%] area of effect"),
    (&[(Stat::Duration, ModifierKind::Flat)], "[{+0}] duration"),
    (&[(Stat::Duration, ModifierKind::Increased)], "[{+0}%] duration"),
    (&[(Stat::PickupRadius, ModifierKind::Flat)], "[{+0}] pickup radius"),
    (&[(Stat::PickupRadius, ModifierKind::Increased)], "[{+0}%] pickup radius"),
    (&[(Stat::AttackSpeed, ModifierKind::Flat)], "[{+0}] attack speed"),
    (&[(Stat::AttackSpeed, ModifierKind::More)], "[{|0|}% more;{|0|}% less] attack speed"),
];

const SNAPSHOT_DISPLAY_RULES: &[(Stat, &str)] = &[
    (Stat::MaxLife, "Max Life: [{0}]"),
    (Stat::MaxMana, "Max Mana: [{0}]"),
    (Stat::PhysicalDamage, "Physical Damage: [{0}]"),
    (Stat::MovementSpeed, "Movement Speed: [{0}]"),
    (Stat::ProjectileSpeed, "Projectile Speed: [{0}]"),
    (Stat::ProjectileCount, "Projectiles: [{0}]"),
    (Stat::CritChance, "Crit Chance: [{0}%]"),
    (Stat::CritMultiplier, "Crit Multiplier: [{0}%]"),
    (Stat::AttackSpeed, "Attack Speed: [{0}%]"),
    (Stat::AreaOfEffect, "Area of Effect: [{0}]"),
    (Stat::Duration, "Duration: [{0}]"),
    (Stat::PickupRadius, "Pickup Radius: [{0}]"),
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

fn key_sort_index(key: &StatKey) -> (usize, usize) {
    (key.0.index(), key.1.index())
}

impl StatDisplayRegistry {
    pub fn build() -> Self {
        let mut single_stat_formats: HashMap<StatKey, Vec<FormatSpan>> = HashMap::new();
        let mut multi_stat_formats: HashMap<Vec<StatKey>, MultiStatEntry> = HashMap::new();

        for (keys, format) in DISPLAY_RULES {
            let spans = parse_format_string(format);
            if keys.len() == 1 {
                single_stat_formats.insert(keys[0], spans);
            } else {
                let mut sorted_key = keys.to_vec();
                sorted_key.sort_by_key(key_sort_index);
                multi_stat_formats.insert(
                    sorted_key,
                    MultiStatEntry {
                        stats: keys.to_vec(),
                        lines: vec![spans],
                    },
                );
            }
        }

        for stat in Stat::iter() {
            for kind in [ModifierKind::Flat, ModifierKind::Increased, ModifierKind::More] {
                let key = (stat, kind);
                if !single_stat_formats.contains_key(&key) {
                    let title = to_title_case(stat.name());
                    let fallback_fmt = format!("[{{+0}}] {}", title);
                    single_stat_formats.insert(key, parse_format_string(&fallback_fmt));
                }
            }
        }

        let mut snapshot_formats: HashMap<Stat, Vec<FormatSpan>> = HashMap::new();
        for (stat, format) in SNAPSHOT_DISPLAY_RULES {
            snapshot_formats.insert(*stat, parse_format_string(format));
        }
        for stat in Stat::iter() {
            if !snapshot_formats.contains_key(&stat) {
                let title = to_title_case(stat.name());
                let fallback_fmt = format!("{}: [{{0}}]", title);
                snapshot_formats.insert(stat, parse_format_string(&fallback_fmt));
            }
        }

        Self {
            single_stat_formats,
            multi_stat_formats,
            snapshot_formats,
        }
    }

    pub fn get_snapshot_format(&self, stat: Stat) -> Option<&[FormatSpan]> {
        self.snapshot_formats.get(&stat).map(|v| v.as_slice())
    }

    pub fn get_format(&self, keys: &[StatKey]) -> Vec<Vec<FormatSpan>> {
        if keys.len() == 1 {
            if let Some(spans) = self.single_stat_formats.get(&keys[0]) {
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

        let mut sorted_key: Vec<StatKey> = keys.to_vec();
        sorted_key.sort_by_key(key_sort_index);

        if let Some(entry) = self.multi_stat_formats.get(&sorted_key) {
            let mut remap = vec![0usize; entry.stats.len()];
            for (rule_pos, key) in entry.stats.iter().enumerate() {
                if let Some(caller_pos) = keys.iter().position(|k| k == key) {
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

        keys.iter()
            .filter_map(|key| self.single_stat_formats.get(key).cloned())
            .collect()
    }
}
