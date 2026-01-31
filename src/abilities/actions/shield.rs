use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::node::{NodeHandler, NodeKind};
use crate::abilities::events::ExecuteNodeEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::wave::InvulnerableStack;
use crate::Faction;
use crate::GameState;

const DEFAULT_SHIELD_DURATION: f32 = 0.5;
const DEFAULT_SHIELD_RADIUS: f32 = 100.0;

#[derive(Component)]
pub struct ShieldActive {
    pub timer: Timer,
    pub radius: f32,
    pub owner_faction: Faction,
}

#[derive(Component)]
pub struct ShieldVisual {
    pub owner: Entity,
}

fn execute_shield_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteNodeEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
    mut invuln_query: Query<&mut InvulnerableStack>,
) {
    let Some(handler_id) = node_registry.get_id("shield") else {
        return;
    };

    for event in action_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };
        let Some(node_def) = ability_def.get_node(event.node_id) else {
            continue;
        };

        if node_def.node_type != handler_id {
            continue;
        }

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();

        let duration = node_def
            .get_f32("duration", &caster_stats, &node_registry)
            .unwrap_or(DEFAULT_SHIELD_DURATION);
        let radius = node_def
            .get_f32("radius", &caster_stats, &node_registry)
            .unwrap_or(DEFAULT_SHIELD_RADIUS);

        let caster = event.context.caster;

        if let Ok(mut entity_commands) = commands.get_entity(caster) {
            entity_commands.insert(ShieldActive {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                radius,
                owner_faction: event.context.caster_faction,
            });

            if let Ok(mut stack) = invuln_query.get_mut(caster) {
                stack.increment();
            } else {
                entity_commands.insert(InvulnerableStack(1));
            }
        }

        commands.spawn((
            Name::new("ShieldVisual"),
            ShieldVisual { owner: caster },
            Sprite {
                color: Color::srgba(0.3, 0.5, 1.0, 0.3),
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            Transform::from_translation(event.context.source_point.with_z(0.5)),
        ));
    }
}

#[derive(Default)]
pub struct ShieldHandler;

impl NodeHandler for ShieldHandler {
    fn name(&self) -> &'static str {
        "shield"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Action
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_shield_action
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_shield, update_shield_visual).in_set(GameSet::AbilityExecution),
        );
    }
}

fn update_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut shield_query: Query<(Entity, &mut ShieldActive, &Transform)>,
    mut invuln_query: Query<&mut InvulnerableStack>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut shield, shield_transform) in &mut shield_query {
        let shield_pos = shield_transform.translation.truncate();

        let projectile_layer = match shield.owner_faction {
            Faction::Player => GameLayer::EnemyProjectile,
            Faction::Enemy => GameLayer::PlayerProjectile,
        };

        let filter = SpatialQueryFilter::from_mask(projectile_layer);
        let shape = Collider::circle(shield.radius);
        let hits = spatial_query.shape_intersections(&shape, shield_pos, 0.0, &filter);

        for proj_entity in hits {
            if let Ok(mut entity_commands) = commands.get_entity(proj_entity) {
                entity_commands.despawn();
            }
        }

        if shield.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).remove::<ShieldActive>();

            if let Ok(mut stack) = invuln_query.get_mut(entity) {
                if stack.decrement() {
                    commands.entity(entity).remove::<InvulnerableStack>();
                }
            }
        }
    }
}

fn update_shield_visual(
    mut commands: Commands,
    shield_query: Query<&Transform, With<ShieldActive>>,
    mut visual_query: Query<(Entity, &ShieldVisual, &mut Transform), Without<ShieldActive>>,
) {
    for (visual_entity, visual, mut visual_transform) in &mut visual_query {
        if let Ok(owner_transform) = shield_query.get(visual.owner) {
            visual_transform.translation = owner_transform.translation.with_z(0.5);
        } else if let Ok(mut entity_commands) = commands.get_entity(visual_entity) {
            entity_commands.despawn();
        }
    }
}

register_node!(ShieldHandler);
