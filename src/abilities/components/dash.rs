use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::wave::InvulnerableStack;
use crate::MovementLocked;
use crate::GameState;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub speed: ScalarExprRaw,
    pub duration: ScalarExprRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub speed: ScalarExpr,
    pub duration: ScalarExpr,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            speed: self.speed.resolve(stat_registry),
            duration: self.duration.resolve(stat_registry),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let fields = raw.speed.required_fields().union(raw.duration.required_fields());
    (fields, None)
}

#[derive(Component)]
pub struct Dash {
    pub speed: f32,
    pub duration: f32,
    pub direction: Vec2,
    pub caster: Entity,
}

#[derive(Component)]
pub struct Dashing {
    pub timer: Timer,
    pub direction: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct PreDashLayers(pub CollisionLayers);

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let speed = def.speed.eval(&eval_ctx);
    let duration = def.duration.eval(&eval_ctx);

    let direction = ctx.target.direction.unwrap_or(Vec2::ZERO);

    if direction != Vec2::ZERO {
        commands.insert(Dash {
            speed,
            duration,
            direction,
            caster: ctx.caster,
        });
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (apply_dash_requests, update_dashing)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn apply_dash_requests(
    mut commands: Commands,
    query: Query<(Entity, &Dash)>,
    mut invuln_query: Query<&mut InvulnerableStack>,
    collision_query: Query<&CollisionLayers>,
) {
    for (entity, request) in &query {
        let current_layers = collision_query
            .get(request.caster)
            .copied()
            .unwrap_or_default();
        let dash_layers = CollisionLayers::new(GameLayer::Player, [GameLayer::Wall]);

        if let Ok(mut caster_commands) = commands.get_entity(request.caster) {
            caster_commands.insert((
                Dashing {
                    timer: Timer::from_seconds(request.duration, TimerMode::Once),
                    direction: request.direction,
                    speed: request.speed,
                },
                MovementLocked,
                PreDashLayers(current_layers),
                dash_layers,
            ));

            if let Ok(mut stack) = invuln_query.get_mut(request.caster) {
                stack.increment();
            } else {
                caster_commands.insert(InvulnerableStack(1));
            }
        }

        commands.entity(entity).despawn();
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
