use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::context::AbilityContext;
use crate::abilities::effect_def::EffectDef;
use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::wave::{add_invulnerability, remove_invulnerability};
use crate::MovementLocked;

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

#[derive(Default)]
pub struct DashHandler;

impl EffectHandler for DashHandler {
    fn name(&self) -> &'static str {
        "dash"
    }

    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let stats = &ctx.stats_snapshot;
        let speed = def.get_f32("speed", stats, registry).unwrap_or(DEFAULT_DASH_SPEED);
        let duration = def.get_f32("duration", stats, registry).unwrap_or(DEFAULT_DASH_DURATION);

        let direction = ctx
            .target_direction
            .map(|d| d.truncate().normalize_or_zero())
            .unwrap_or(Vec2::ZERO);

        if direction == Vec2::ZERO {
            return;
        }

        let caster = ctx.caster;

        commands.queue(move |world: &mut World| {
            let current_layers = world
                .get::<CollisionLayers>(caster)
                .copied()
                .unwrap_or_default();

            let dash_layers = CollisionLayers::new(GameLayer::Player, [GameLayer::Wall]);

            if let Ok(mut entity_mut) = world.get_entity_mut(caster) {
                entity_mut.insert((
                    Dashing {
                        timer: Timer::from_seconds(duration, TimerMode::Once),
                        direction,
                        speed,
                    },
                    MovementLocked,
                    PreDashLayers(current_layers),
                    dash_layers,
                ));
            }
        });

        add_invulnerability(commands, caster);
    }

    fn register_systems(&self, app: &mut App) {
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
) {
    for (entity, mut dashing, mut velocity, pre_dash_layers) in &mut query {
        velocity.0 = dashing.direction * dashing.speed;

        if dashing.timer.tick(time.delta()).just_finished() {
            let restored_layers = pre_dash_layers.0;
            commands
                .entity(entity)
                .remove::<(Dashing, MovementLocked, PreDashLayers)>()
                .insert(restored_layers);
            remove_invulnerability(&mut commands, entity);
        }
    }
}

register_effect!(DashHandler);
