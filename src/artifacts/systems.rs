use bevy::prelude::*;
use rand::prelude::*;

use crate::money::PlayerMoney;
use crate::player::Player;
use crate::stats::{Modifiers, StatRange};

use super::registry::ArtifactRegistry;
use super::resources::{Artifact, AvailableArtifacts, PlayerArtifacts, ShopOfferings};

#[derive(Message)]
pub struct RerollRequest;

#[derive(Message)]
pub struct BuyRequest {
    pub index: usize,
}

#[derive(Message)]
pub struct SellRequest {
    pub slot: usize,
}

fn reroll_offerings(
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

pub fn handle_reroll(
    mut commands: Commands,
    mut events: MessageReader<RerollRequest>,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
) {
    for _ in events.read() {
        reroll_offerings(&mut commands, &mut offerings, &available);
    }
}

pub fn handle_buy(
    mut events: MessageReader<BuyRequest>,
    mut money: ResMut<PlayerMoney>,
    mut artifacts: ResMut<PlayerArtifacts>,
    mut offerings: ResMut<ShopOfferings>,
    registry: Res<ArtifactRegistry>,
    artifact_query: Query<&Artifact>,
    mut player_query: Query<&mut Modifiers, With<Player>>,
) {
    for event in events.read() {
        if event.index >= offerings.0.len() {
            continue;
        }
        let artifact_entity = offerings.0[event.index];
        let Ok(artifact) = artifact_query.get(artifact_entity) else {
            continue;
        };
        let Some(def) = registry.get(artifact.0) else {
            continue;
        };
        if money.0 >= def.price && !artifacts.is_full() {
            money.0 -= def.price;
            artifacts.buy(artifact_entity);
            offerings.0.remove(event.index);
            for mut modifiers in &mut player_query {
                for modifier_def in &def.modifiers {
                    for sr in &modifier_def.stats {
                        match sr {
                            StatRange::Fixed { stat, value } => {
                                modifiers.add(*stat, *value, Some(artifact_entity));
                            }
                            StatRange::Range { stat, min, max } => {
                                modifiers.add(*stat, (*min + *max) / 2.0, Some(artifact_entity));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_sell(
    mut commands: Commands,
    mut events: MessageReader<SellRequest>,
    mut money: ResMut<PlayerMoney>,
    mut artifacts: ResMut<PlayerArtifacts>,
    registry: Res<ArtifactRegistry>,
    artifact_query: Query<&Artifact>,
    mut player_query: Query<&mut Modifiers, With<Player>>,
) {
    for event in events.read() {
        if let Some(artifact_entity) = artifacts.sell(event.slot) {
            if let Ok(artifact) = artifact_query.get(artifact_entity) {
                if let Some(def) = registry.get(artifact.0) {
                    money.0 += (def.price + 1) / 2;
                }
            }
            for mut modifiers in &mut player_query {
                modifiers.remove_by_source(artifact_entity);
            }
            commands.entity(artifact_entity).despawn();
        }
    }
}

pub fn generate_shop_offerings(
    mut commands: Commands,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
) {
    reroll_offerings(&mut commands, &mut offerings, &available);
}
