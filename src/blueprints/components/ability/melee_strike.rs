use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::size::Size;
use crate::blueprints::SpawnSource;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::PendingDamage;
use crate::Faction;

#[blueprint_component]
pub struct MeleeStrike {
    pub range: ScalarExpr,
    pub damage: ScalarExpr,
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
    query: Query<(Entity, &MeleeStrike, &SpawnSource)>,
    transforms: Query<&Transform>,
    sizes: Query<&Size>,
    spatial_query: SpatialQuery,
) {
    for (entity, strike, source) in &query {
        let Some(caster_entity) = source.caster.entity else { continue };
        let Ok(caster_transform) = transforms.get(caster_entity) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = caster_transform.translation.truncate();
        let caster_radius = sizes.get(caster_entity).map_or(0.0, |s| s.value / 2.0);

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(strike.range + caster_radius);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        for target_entity in hits {
            pending.write(PendingDamage { target: target_entity, amount: strike.damage, source: Some(caster_entity) });
        }

        commands.entity(entity).despawn();
    }
}
