use bevy::prelude::*;

use crate::artifacts::{ArtifactId, ArtifactRegistry};
use crate::game_state::GameState;
use crate::stats::{StatDisplayRegistry, StatRange};

#[derive(Component)]
pub struct ArtifactTooltipTarget(pub ArtifactId);

#[derive(Component)]
pub struct ArtifactTooltip;

const TOOLTIP_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.95);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const POSITIVE_COLOR: Color = Color::srgb(0.3, 0.9, 0.3);
const NEGATIVE_COLOR: Color = Color::srgb(0.9, 0.3, 0.3);

pub fn update_artifact_tooltip(
    mut commands: Commands,
    tooltip_targets: Query<(Entity, &Interaction, &ArtifactTooltipTarget, &UiGlobalTransform, &ComputedNode)>,
    existing_tooltip: Query<Entity, With<ArtifactTooltip>>,
    registry: Res<ArtifactRegistry>,
    display: Res<StatDisplayRegistry>,
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

    for modifier_def in &def.modifiers {
        let pairs: Vec<_> = modifier_def.stats.iter().map(|sr| match sr {
            StatRange::Fixed { stat, value } => (*stat, *value),
            StatRange::Range { stat, min, max } => (*stat, (*min + *max) / 2.0),
        }).collect();
        let lines = display.format(&pairs);
        for line in lines {
            let value = pairs.first().map(|(_, v)| *v).unwrap_or(0.0);
            let color = if value > 0.0 { POSITIVE_COLOR } else { NEGATIVE_COLOR };

            let row = commands.spawn((
                Text::new(line),
                TextFont { font_size: 16.0, ..default() },
                TextColor(color),
                Node {
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
            )).id();
            commands.entity(tooltip).add_child(row);
        }
    }
}
