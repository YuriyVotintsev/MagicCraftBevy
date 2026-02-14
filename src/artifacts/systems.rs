use bevy::prelude::*;
use rand::prelude::*;

use super::resources::{Artifact, AvailableArtifacts, ShopOfferings};

pub fn reroll_offerings(
    commands: &mut Commands,
    offerings: &mut ShopOfferings,
    available: &AvailableArtifacts,
) {
    for entity in offerings.0.drain(..) {
        commands.entity(entity).despawn();
    }

    let mut rng = rand::rng();
    let mut ids = available.0.clone();
    ids.shuffle(&mut rng);
    offerings.0 = ids
        .into_iter()
        .take(3)
        .map(|id| commands.spawn(Artifact(id)).id())
        .collect();
}

pub fn generate_shop_offerings(
    mut commands: Commands,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
) {
    reroll_offerings(&mut commands, &mut offerings, &available);
}
