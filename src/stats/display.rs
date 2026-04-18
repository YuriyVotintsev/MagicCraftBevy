use std::collections::HashMap;

use bevy::prelude::*;

use super::registry::Stat;

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

#[derive(Resource, Default)]
pub struct StatDisplayRegistry {
    snapshot_formats: HashMap<Stat, Vec<FormatSpan>>,
}

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

impl StatDisplayRegistry {
    pub fn build() -> Self {
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

        Self { snapshot_formats }
    }

    pub fn get_snapshot_format(&self, stat: Stat) -> Option<&[FormatSpan]> {
        self.snapshot_formats.get(&stat).map(|v| v.as_slice())
    }
}
