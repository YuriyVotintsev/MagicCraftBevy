use bevy::prelude::*;

use crate::artifacts::{ArtifactId, ArtifactRegistry};
use crate::game_state::GameState;

#[derive(Component)]
pub struct ArtifactTooltipTarget(pub ArtifactId);

#[derive(Component)]
pub struct ArtifactTooltip;

const TOOLTIP_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.95);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const POSITIVE_COLOR: Color = Color::srgb(0.3, 0.9, 0.3);
const NEGATIVE_COLOR: Color = Color::srgb(0.9, 0.3, 0.3);

fn format_stat_name(raw: &str) -> String {
    let name = raw
        .strip_suffix("_base")
        .or_else(|| raw.strip_suffix("_more"))
        .or_else(|| raw.strip_suffix("_increased"))
        .unwrap_or(raw);
    name.split('_')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_modifier_value(value: f32) -> String {
    if value > 0.0 {
        if value.fract() == 0.0 {
            format!("+{}", value as i32)
        } else {
            format!("+{:.2}", value)
        }
    } else if value.fract() == 0.0 {
        format!("{}", value as i32)
    } else {
        format!("{:.2}", value)
    }
}

pub fn update_artifact_tooltip(
    mut commands: Commands,
    tooltip_targets: Query<(Entity, &Interaction, &ArtifactTooltipTarget, &UiGlobalTransform, &ComputedNode)>,
    existing_tooltip: Query<Entity, With<ArtifactTooltip>>,
    registry: Res<ArtifactRegistry>,
    mut last_hovered: Local<Option<(ArtifactId, Entity)>>,
) {
    let mut hovered = None;
    let mut hovered_center = Vec2::ZERO;
    let mut hovered_size = Vec2::ZERO;
    let mut inv_scale = 1.0_f32;
    for (entity, interaction, target, gt, cn) in &tooltip_targets {
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            hovered = Some((target.0, entity));
            hovered_center = gt.translation;
            hovered_size = cn.size();
            inv_scale = cn.inverse_scale_factor();
            break;
        }
    }

    if hovered == *last_hovered {
        return;
    }
    *last_hovered = hovered;

    for entity in &existing_tooltip {
        commands.entity(entity).despawn();
    }

    let Some((artifact_id, _)) = hovered else {
        return;
    };
    let Some(def) = registry.get(artifact_id) else {
        return;
    };
    if def.modifiers.is_empty() {
        return;
    }

    const TOOLTIP_WIDTH: f32 = 260.0;
    let top_left = (hovered_center - hovered_size / 2.0) * inv_scale;
    let left = (top_left.x - TOOLTIP_WIDTH - 8.0).max(0.0);
    let top = top_left.y.max(0.0);

    let tooltip = commands
        .spawn((
            ArtifactTooltip,
            DespawnOnExit(GameState::Playing),
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(12.0)),
                min_width: Val::Px(220.0),
                ..default()
            },
            BackgroundColor(TOOLTIP_BG),
        ))
        .id();

    let header = commands
        .spawn((
            Text::new(&def.name),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(tooltip).add_child(header);

    let mut sorted_mods: Vec<_> = def.modifiers.iter().collect();
    sorted_mods.sort_by(|a, b| a.name.cmp(&b.name));

    for modifier in sorted_mods {
        let display_name = format_stat_name(&modifier.name);
        let value_str = format_modifier_value(modifier.value);
        let color = if modifier.value > 0.0 {
            POSITIVE_COLOR
        } else {
            NEGATIVE_COLOR
        };

        let row = commands
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            })
            .id();

        let name_text = commands
            .spawn((
                Text::new(display_name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ))
            .id();

        let value_text = commands
            .spawn((
                Text::new(value_str),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::left(Val::Px(20.0)),
                    ..default()
                },
            ))
            .id();

        commands.entity(row).add_child(name_text);
        commands.entity(row).add_child(value_text);
        commands.entity(tooltip).add_child(row);
    }
}
