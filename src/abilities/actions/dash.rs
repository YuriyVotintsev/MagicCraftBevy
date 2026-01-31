use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::registry::{ActionHandler, ActionRegistry};
use crate::abilities::events::ExecuteActionEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::wave::InvulnerableStack;
use crate::MovementLocked;
use crate::GameState;

const DEFAULT_DASH_SPEED: f32 = 1500.0;
const DEFAULT_DASH_DURATION: f32 = 0.2;

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
    mut action_events: MessageReader<ExecuteActionEvent>,
    action_registry: Res<ActionRegistry>,
    stats_query: Query<&ComputedStats>,
    mut invuln_query: Query<&mut InvulnerableStack>,
    collision_query: Query<&CollisionLayers>,
) {
    for event in action_events.read() {
        let Some(handler_id) = action_registry.get_id("dash") else {
            continue;
        };
        if event.action.action_type != handler_id {
            continue;
        }

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();

        let speed = event
            .action
            .get_f32("speed", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_DASH_SPEED);
        let duration = event
            .action
            .get_f32("duration", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_DASH_DURATION);

        let direction = event
            .context
            .target_direction
            .map(|d| d.truncate().normalize_or_zero())
            .unwrap_or(Vec2::ZERO);

        if direction == Vec2::ZERO {
            continue;
        }

        let caster = event.context.caster;

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

#[derive(Default)]
pub struct DashHandler;

impl ActionHandler for DashHandler {
    fn name(&self) -> &'static str {
        "dash"
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_dash_action
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_dashing.in_set(GameSet::AbilityExecution),
        );
    }
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

register_action!(DashHandler);
