use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::ParamValue;
use crate::building_blocks::actions::ExecuteShieldEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::wave::InvulnerableStack;
use crate::Faction;
use crate::GameState;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Action)]
pub struct ShieldParams {
    #[raw(default = 0.5)]
    pub duration: ParamValue,
    #[raw(default = 100.0)]
    pub radius: ParamValue,
}

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
    mut action_events: MessageReader<ExecuteShieldEvent>,
    stats_query: Query<&ComputedStats>,
    mut invuln_query: Query<&mut InvulnerableStack>,
) {
    for event in action_events.read() {
        let caster_stats = stats_query
            .get(event.base.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let duration = event.params.duration.evaluate_f32(&caster_stats);
        let radius = event.params.radius.evaluate_f32(&caster_stats);

        let caster = event.base.context.caster;

        if let Ok(mut entity_commands) = commands.get_entity(caster) {
            entity_commands.insert(ShieldActive {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                radius,
                owner_faction: event.base.context.caster_faction,
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
            Transform::from_translation(event.base.context.source.as_point().unwrap_or(Vec3::ZERO).with_z(0.5)),
        ));
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        execute_shield_action
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        (update_shield, update_shield_visual).in_set(GameSet::AbilityExecution),
    );
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

register_node!(ShieldParams);
