use bevy::prelude::*;
use rand::prelude::*;

use crate::balance::GameBalance;
use crate::money::PlayerMoney;
use crate::player::Player;
use crate::stats::Modifiers;

use super::registry::ArtifactRegistry;
use super::resources::{AvailableArtifacts, PlayerArtifacts, RerollCost, ShopOfferings};
use super::types::{Artifact, ArtifactEntity};

#[derive(Message)]
pub struct RerollRequest;

#[derive(Message)]
pub struct BuyRequest {
    pub artifact: ArtifactEntity,
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

    // By design: duplicates allowed — stacking works via stat modifiers
    let mut rng = rand::rng();
    let pool = available.as_slice();
    offerings.set(
        (0..offerings_count)
            .map(|_| {
                commands
                    .spawn(Artifact {
                        artifact_id: *pool.choose(&mut rng).unwrap(),
                        values: vec![],
                    })
                    .id()
            })
            .collect(),
    );
}

pub fn handle_reroll(
    mut commands: Commands,
    mut events: MessageReader<RerollRequest>,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
    mut money: ResMut<PlayerMoney>,
    mut reroll_cost: ResMut<RerollCost>,
    balance: If<Res<GameBalance>>,
) {
    for _ in events.read() {
        if money.spend(reroll_cost.get()) {
            reroll_cost.increment();
            reroll_offerings(&mut commands, &mut offerings, &available, balance.shop.offerings_count);
        }
    }
}

pub fn handle_buy(
    mut events: MessageReader<BuyRequest>,
    mut money: ResMut<PlayerMoney>,
    mut artifacts: ResMut<PlayerArtifacts>,
    mut offerings: ResMut<ShopOfferings>,
    registry: Res<ArtifactRegistry>,
    mut artifact_query: Query<&mut Artifact>,
    mut player_query: Query<&mut Modifiers, With<Player>>,
) {
    for event in events.read() {
        let artifact_entity = event.artifact.0;
        let Some(index) = offerings.position(artifact_entity) else {
            continue;
        };
        let Ok(mut artifact) = artifact_query.get_mut(artifact_entity) else {
            continue;
        };
        let artifact_id = artifact.artifact_id;
        let Some(def) = registry.get(artifact_id) else {
            continue;
        };
        if !artifacts.is_full() && money.spend(def.price) {
            let mut rng = rand::rng();
            artifact.values = def.roll_values(&mut rng);
            artifacts.buy(artifact_entity);
            offerings.remove(index);
            for mut modifiers in &mut player_query {
                for &(stat, value) in &artifact.values {
                    modifiers.add(stat, value, Some(artifact_entity));
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
                if let Some(def) = registry.get(artifact.artifact_id) {
                    money.earn(def.sell_price(balance.shop.sell_price_percent));
                }
            }
            for mut modifiers in &mut player_query {
                modifiers.remove_by_source(artifact_entity);
            }
            commands.entity(artifact_entity).try_despawn();
        }
    }
}

pub fn reset_shop(
    mut commands: Commands,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
    mut reroll_cost: ResMut<RerollCost>,
    balance: Res<GameBalance>,
) {
    reroll_cost.reset_to(balance.shop.base_reroll_cost);
    reroll_offerings(&mut commands, &mut offerings, &available, balance.shop.offerings_count);
}
