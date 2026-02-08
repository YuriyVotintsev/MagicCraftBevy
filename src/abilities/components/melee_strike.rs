use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::AbilitySource;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::PendingDamage;
use crate::Faction;

#[ability_component]
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
    query: Query<(Entity, &MeleeStrike, &AbilitySource)>,
    transforms: Query<&Transform>,
    spatial_query: SpatialQuery,
) {
    for (entity, strike, source) in &query {
        let Some(caster_entity) = source.caster.entity else { continue };
        let Ok(caster_transform) = transforms.get(caster_entity) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = caster_transform.translation.truncate();

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(strike.range);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        for target_entity in hits {
            if let Ok(mut target_commands) = commands.get_entity(target_entity) {
                target_commands.insert(PendingDamage(strike.damage));
            }
        }

        commands.entity(entity).despawn();
    }
}
