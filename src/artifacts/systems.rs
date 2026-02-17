use bevy::prelude::*;
use rand::prelude::*;

use crate::balance::GameBalance;
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
    offerings_count: usize,
) {
    offerings.clear(commands);

    let mut rng = rand::rng();
    let mut ids = available.to_vec();
    ids.shuffle(&mut rng);
    offerings.set(
        ids.into_iter()
            .take(offerings_count)
            .map(|id| commands.spawn(Artifact(id)).id())
            .collect(),
    );
}

pub fn handle_reroll(
    mut commands: Commands,
    mut events: MessageReader<RerollRequest>,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
    balance: If<Res<GameBalance>>,
) {
    for _ in events.read() {
        reroll_offerings(&mut commands, &mut offerings, &available, balance.shop.offerings_count);
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
        if event.index >= offerings.len() {
            continue;
        }
        let Some(artifact_entity) = offerings.get(event.index) else {
            continue;
        };
        let Ok(artifact) = artifact_query.get(artifact_entity) else {
            continue;
        };
        let Some(def) = registry.get(artifact.0) else {
            continue;
        };
        if !artifacts.is_full() && money.spend(def.price) {
            artifacts.buy(artifact_entity);
            offerings.remove(event.index);
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
    balance: If<Res<GameBalance>>,
) {
    for event in events.read() {
        if let Some(artifact_entity) = artifacts.sell(event.slot) {
            if let Ok(artifact) = artifact_query.get(artifact_entity) {
                if let Some(def) = registry.get(artifact.0) {
                    money.earn(def.sell_price(balance.shop.sell_price_percent));
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
    balance: Res<GameBalance>,
) {
    reroll_offerings(&mut commands, &mut offerings, &available, balance.shop.offerings_count);
}
