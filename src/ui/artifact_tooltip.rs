use bevy::prelude::*;

use crate::artifacts::{Artifact, ArtifactRegistry};
use crate::game_state::GameState;
use crate::stats::{StatDisplayRegistry, StatRange};

use super::stat_line_builder::{StatLineBuilder, StatRenderMode, GOLD_COLOR};

#[derive(Component)]
pub struct ArtifactTooltipTarget(pub Entity);

#[derive(Component)]
pub struct ArtifactTooltip;

const TOOLTIP_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.95);

pub fn update_artifact_tooltip(
    mut commands: Commands,
    tooltip_targets: Query<(Entity, &Interaction, &ArtifactTooltipTarget, &UiGlobalTransform, &ComputedNode)>,
    existing_tooltip: Query<Entity, With<ArtifactTooltip>>,
    artifact_query: Query<&Artifact>,
    registry: Res<ArtifactRegistry>,
    display: Res<StatDisplayRegistry>,
    mut last_hovered: Local<Option<Entity>>,
) {
    let mut hovered = None;
    let mut hovered_artifact = None;
    let mut hovered_center = Vec2::ZERO;
    let mut hovered_size = Vec2::ZERO;
    let mut inv_scale = 1.0_f32;
    for (entity, interaction, target, gt, cn) in &tooltip_targets {
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            hovered = Some(entity);
            hovered_artifact = Some(target.0);
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

    let Some(artifact_entity) = hovered_artifact else {
        return;
    };
    let Ok(artifact) = artifact_query.get(artifact_entity) else {
        return;
    };
    let Some(def) = registry.get(artifact.artifact_id) else {
        return;
    };
    if def.modifiers.is_empty() && artifact.values.is_empty() {
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

    if !artifact.values.is_empty() {
        for (stat, value) in &artifact.values {
            let formats = display.get_format(&[*stat]);
            let row = StatLineBuilder::spawn_line(
                &mut commands,
                &formats[0],
                StatRenderMode::Fixed { values: &[*value] },
                16.0,
            );
            commands.entity(row).insert(Node {
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            });
            commands.entity(tooltip).add_child(row);
        }
    } else {
        for m in &def.modifiers {
            let has_range = m.stats.iter().any(|sr| matches!(sr, StatRange::Range { .. }));
            let stat_ids: Vec<_> = m.stats.iter().map(|sr| match sr {
                StatRange::Fixed { stat, .. } | StatRange::Range { stat, .. } => *stat,
            }).collect();
            let formats = display.get_format(&stat_ids);

            if has_range {
                let ranges: Vec<(f32, f32)> = m.stats.iter().map(|sr| match sr {
                    StatRange::Fixed { value, .. } => (*value, *value),
                    StatRange::Range { min, max, .. } => (*min, *max),
                }).collect();
                for line in &formats {
                    let row = StatLineBuilder::spawn_line(
                        &mut commands, line,
                        StatRenderMode::Range { ranges: &ranges },
                        16.0,
                    );
                    commands.entity(row).insert(Node {
                        margin: UiRect::vertical(Val::Px(2.0)),
                        ..default()
                    });
                    commands.entity(tooltip).add_child(row);
                }
            } else {
                let values: Vec<f32> = m.stats.iter().map(|sr| match sr {
                    StatRange::Fixed { value, .. } => *value,
                    StatRange::Range { min, max, .. } => (*min + *max) / 2.0,
                }).collect();
                for line in &formats {
                    let row = StatLineBuilder::spawn_line(
                        &mut commands, line,
                        StatRenderMode::Fixed { values: &values },
                        16.0,
                    );
                    commands.entity(row).insert(Node {
                        margin: UiRect::vertical(Val::Px(2.0)),
                        ..default()
                    });
                    commands.entity(tooltip).add_child(row);
                }
            }
        }
    }
}
