use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::size::Size;
use crate::movement::SelfMoving;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, PendingDamage, StatRegistry};
use crate::Faction;

#[blueprint_component]
pub struct SpinnerCharge {
    pub speed: ScalarExpr,
    #[raw(default = 2.0)]
    pub max_duration: ScalarExpr,
    #[raw(default = 150.0)]
    pub hit_radius: ScalarExpr,
    #[default_expr("target.entity")]
    pub target: EntityExpr,
}

#[derive(Component)]
pub struct SpinnerChargeState {
    pub elapsed: f32,
    pub hit_player: bool,
}

#[derive(Component)]
pub struct PreChargeLayers(pub CollisionLayers);

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_charge, charge_system)
            .chain()
            .in_set(GameSet::MobAI),
    );
    app.add_observer(on_remove_charge);
}

fn init_charge(
    mut commands: Commands,
    query: Query<(Entity, &SpinnerCharge, &Transform), Added<SpinnerCharge>>,
    target_transforms: Query<&Transform, Without<SpinnerCharge>>,
    collision_query: Query<&CollisionLayers>,
) {
    for (entity, charge, transform) in &query {
        let target_pos = target_transforms
            .get(charge.target)
            .map(|t| crate::coord::to_2d(t.translation))
            .unwrap_or_default();

        let my_pos = crate::coord::to_2d(transform.translation);
        let diff = target_pos - my_pos;
        let direction = if diff.length_squared() > 1.0 {
            diff.normalize()
        } else {
            Vec2::X
        };

        let current_layers = collision_query
            .get(entity)
            .copied()
            .unwrap_or_default();

        let charge_layers = CollisionLayers::new(
            GameLayer::Enemy,
            [GameLayer::Wall, GameLayer::PlayerProjectile],
        );

        commands.entity(entity).insert((
            SpinnerChargeState {
                elapsed: 0.0,
                hit_player: false,
            },
            PreChargeLayers(current_layers),
            charge_layers,
            SelfMoving,
            LinearVelocity(crate::coord::ground_vel(direction * charge.speed)),
        ));

    }
}

fn charge_system(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &SpinnerCharge,
        &mut SpinnerChargeState,
        &Transform,
        &LinearVelocity,
        &Faction,
        Option<&Size>,
    )>,
    stat_registry: Option<Res<StatRegistry>>,
    stats_query: Query<&ComputedStats>,
    spatial_query: SpatialQuery,
    mut pending: MessageWriter<PendingDamage>,
) {
    let dt = time.delta_secs();

    for (entity, charge, mut state, transform, velocity, faction, size) in &mut query {
        state.elapsed += dt;

        if *faction == Faction::Enemy && !state.hit_player {
            let position = crate::coord::to_2d(transform.translation);
            let entity_radius = size.map_or(0.0, |s| s.value / 2.0);

            let damage = stat_registry
                .as_ref()
                .and_then(|sr| sr.get("physical_damage_flat"))
                .and_then(|id| stats_query.get(entity).ok().map(|s| s.get(id)))
                .unwrap_or(10.0);

            let filter = SpatialQueryFilter::from_mask(GameLayer::Player);
            let shape = Collider::sphere(charge.hit_radius + entity_radius);
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
                state.hit_player = true;
            }
        }

        let vel_2d = crate::coord::to_2d(velocity.0);
        let stopped = vel_2d.length_squared() < 1.0 && state.elapsed > 0.1;
        let timed_out = state.elapsed >= charge.max_duration;

        if stopped || timed_out {
            // AfterTime transition will handle the actual state change,
            // but we can also force it for wall-stop case
        }
    }
}

fn on_remove_charge(
    on: On<Remove, SpinnerCharge>,
    mut commands: Commands,
    layers_query: Query<&PreChargeLayers>,
    mut velocity_query: Query<&mut LinearVelocity>,
) {
    let entity = on.event_target();
    let restored = layers_query.get(entity).ok().map(|pre| pre.0);

    if let Ok(mut vel) = velocity_query.get_mut(entity) {
        vel.0 = Vec3::ZERO;
    }

    commands
        .entity(entity)
        .queue_silenced(move |mut e: EntityWorldMut| {
            if let Some(layers) = restored {
                e.insert(layers);
            }
            e.remove::<(SpinnerChargeState, PreChargeLayers, SelfMoving)>();
        });
}
