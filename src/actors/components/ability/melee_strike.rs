use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::components::common::size::Size;
use crate::actors::SpawnSource;
use crate::schedule::GameSet;
use crate::actors::combat::PendingDamage;
use crate::Faction;

#[derive(Component)]
pub struct MeleeStrike {
    pub range: f32,
    pub damage: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        melee_strike_system.in_set(GameSet::Damage),
    );
}

fn melee_strike_system(
    mut commands: Commands,
    mut pending: MessageWriter<PendingDamage>,
    query: Query<(Entity, &MeleeStrike, &SpawnSource, &Faction)>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
    spatial_query: SpatialQuery,
) {
    for (entity, strike, source, faction) in &query {
        let Some(caster_entity) = source.caster.entity else { continue };
        let Ok(caster_transform) = transforms.get(caster_entity) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = crate::coord::to_2d(caster_transform.translation);
        let caster_radius = sizes.get(caster_entity).map_or(0.0, |s| s.value / 2.0);

        let filter = SpatialQueryFilter::from_mask(faction.enemy_layer());
        let shape = Collider::sphere(strike.range + caster_radius);
        let hits = spatial_query.shape_intersections(&shape, crate::coord::ground_pos(position), Quat::IDENTITY, &filter);

        for target_entity in hits {
            pending.write(PendingDamage { target: target_entity, amount: strike.damage, source: Some(caster_entity) });
        }

        commands.entity(entity).despawn();
    }
}
