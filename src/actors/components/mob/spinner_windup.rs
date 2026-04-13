use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::components::common::size::Size;
use crate::actors::components::common::spinner_visual::SpinnerVisualState;
use crate::particles;
use crate::actors::components::common::collider::GameLayer;
use crate::schedule::GameSet;
use crate::actors::combat::PendingDamage;
use crate::stats::{ComputedStats, Stat};
use crate::Faction;

#[derive(Component)]
pub struct SpinnerWindup {
    pub duration: f32,
    pub max_spin_speed: f32,
    pub spike_growth_max: f32,
    pub squish_min: f32,
    pub contact_radius: f32,
}

#[derive(Component)]
pub struct SpinnerWindupTimer {
    pub elapsed: f32,
    pub damage_cooldown: f32,
}

const DAMAGE_INTERVAL: f32 = 0.25;
const TRAIL_THRESHOLD: f32 = 0.5;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_windup, windup_system)
            .chain()
            .in_set(GameSet::MobAI),
    );
    app.add_observer(on_remove_windup);
}

fn init_windup(mut commands: Commands, query: Query<Entity, Added<SpinnerWindup>>) {
    for entity in &query {
        commands.entity(entity).insert(SpinnerWindupTimer {
            elapsed: 0.0,
            damage_cooldown: 0.0,
        });
    }
}

fn windup_system(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &SpinnerWindup,
        &mut SpinnerWindupTimer,
        &mut SpinnerVisualState,
        &Transform,
        &Faction,
        Option<&Size>,
    )>,
    stats_query: Query<&ComputedStats>,
    spatial_query: SpatialQuery,
    mut pending: MessageWriter<PendingDamage>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, windup, mut timer, mut visual_state, transform, faction, size) in &mut query {
        timer.elapsed += dt;
        let t = (timer.elapsed / windup.duration).clamp(0.0, 1.0);
        let ease = t * t;

        visual_state.spin_speed = windup.max_spin_speed * ease;
        visual_state.spike_growth = 1.0 + (windup.spike_growth_max - 1.0) * t;
        visual_state.squish = 1.0 + (windup.squish_min - 1.0) * t;

        if t > TRAIL_THRESHOLD {
            let pos = crate::coord::to_2d(transform.translation);
            for i in 0..visual_state.trail_emitters.len() {
                if visual_state.trail_emitters[i].is_none() {
                    let emitter_entity = particles::start_particles(&mut commands, "spinner_trail", pos);
                    visual_state.trail_emitters[i] = Some(emitter_entity);
                }
            }
        }

        if *faction != Faction::Enemy {
            continue;
        }
        timer.damage_cooldown -= dt;
        if timer.damage_cooldown > 0.0 {
            continue;
        }

        let position = crate::coord::to_2d(transform.translation);
        let entity_radius = size.map_or(0.0, |s| s.value / 2.0);

        let damage = stats_query
            .get(entity)
            .map(|s| s.get(Stat::PhysicalDamageFlat))
            .unwrap_or(windup.contact_radius);

        let filter = SpatialQueryFilter::from_mask(GameLayer::Player);
        let shape = Collider::sphere(windup.contact_radius + entity_radius);
        let hits = spatial_query.shape_intersections(
            &shape,
            crate::coord::ground_pos(position),
            Quat::IDENTITY,
            &filter,
        );

        for target in hits {
            pending.write(PendingDamage {
                target,
                amount: damage,
                source: Some(entity),
            });
        }
        timer.damage_cooldown = DAMAGE_INTERVAL;
    }
}

fn on_remove_windup(
    on: On<Remove, SpinnerWindup>,
    mut commands: Commands,
) {
    let entity = on.event_target();
    commands
        .entity(entity)
        .queue_silenced(|mut e: EntityWorldMut| {
            e.remove::<SpinnerWindupTimer>();
        });
}
