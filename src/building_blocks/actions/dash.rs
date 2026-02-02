use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::ParamValue;
use crate::abilities::Target;
use crate::building_blocks::actions::ExecuteDashEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::wave::InvulnerableStack;
use crate::MovementLocked;
use crate::GameState;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Action)]
pub struct DashParams {
    #[raw(default = 1500.0)]
    pub speed: ParamValue,
    #[raw(default = 0.2)]
    pub duration: ParamValue,
}

#[derive(Component)]
pub struct Dashing {
    pub timer: Timer,
    pub direction: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct PreDashLayers(pub CollisionLayers);

fn execute_dash_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteDashEvent>,
    stats_query: Query<&ComputedStats>,
    mut invuln_query: Query<&mut InvulnerableStack>,
    collision_query: Query<&CollisionLayers>,
) {
    for event in action_events.read() {
        let caster_stats = stats_query
            .get(event.base.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let speed = event.params.speed.evaluate_f32(&caster_stats);
        let duration = event.params.duration.evaluate_f32(&caster_stats);

        let direction = match event.base.context.target {
            Some(Target::Direction(d)) => d.truncate().normalize_or_zero(),
            _ => Vec2::ZERO,
        };

        if direction == Vec2::ZERO {
            continue;
        }

        let caster = event.base.context.caster;

        let current_layers = collision_query
            .get(caster)
            .copied()
            .unwrap_or_default();
        let dash_layers = CollisionLayers::new(GameLayer::Player, [GameLayer::Wall]);

        if let Ok(mut entity_commands) = commands.get_entity(caster) {
            entity_commands.insert((
                Dashing {
                    timer: Timer::from_seconds(duration, TimerMode::Once),
                    direction,
                    speed,
                },
                MovementLocked,
                PreDashLayers(current_layers),
                dash_layers,
            ));

            if let Ok(mut stack) = invuln_query.get_mut(caster) {
                stack.increment();
            } else {
                entity_commands.insert(InvulnerableStack(1));
            }
        }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        execute_dash_action
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        update_dashing.in_set(GameSet::AbilityExecution),
    );
}

fn update_dashing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dashing, &mut LinearVelocity, &PreDashLayers)>,
    mut invuln_query: Query<&mut InvulnerableStack>,
) {
    for (entity, mut dashing, mut velocity, pre_dash_layers) in &mut query {
        velocity.0 = dashing.direction * dashing.speed;

        if dashing.timer.tick(time.delta()).just_finished() {
            let restored_layers = pre_dash_layers.0;
            commands
                .entity(entity)
                .remove::<(Dashing, MovementLocked, PreDashLayers)>()
                .insert(restored_layers);

            if let Ok(mut stack) = invuln_query.get_mut(entity) {
                if stack.decrement() {
                    commands.entity(entity).remove::<InvulnerableStack>();
                }
            }
        }
    }
}

register_node!(DashParams);
