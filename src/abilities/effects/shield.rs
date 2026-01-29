use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::context::AbilityContext;
use crate::abilities::effect_def::EffectDef;
use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::wave::{add_invulnerability, remove_invulnerability};
use crate::Faction;

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

#[derive(Default)]
pub struct ShieldHandler;

impl EffectHandler for ShieldHandler {
    fn name(&self) -> &'static str {
        "shield"
    }

    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let stats = &ctx.stats_snapshot;
        let duration = def.get_f32("duration", stats, registry).unwrap_or(DEFAULT_SHIELD_DURATION);
        let radius = def.get_f32("radius", stats, registry).unwrap_or(DEFAULT_SHIELD_RADIUS);

        let caster = ctx.caster;
        let caster_position = ctx.caster_position;

        commands.entity(caster).insert(ShieldActive {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            radius,
            owner_faction: ctx.caster_faction,
        });

        add_invulnerability(commands, caster);

        commands.spawn((
            Name::new("ShieldVisual"),
            ShieldVisual { owner: caster },
            Sprite {
                color: Color::srgba(0.3, 0.7, 1.0, 0.4),
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            Transform::from_translation(caster_position.with_z(0.5)),
        ));
    }

    fn register_systems(&self, app: &mut App) {
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
            remove_invulnerability(&mut commands, entity);
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

register_effect!(ShieldHandler);
