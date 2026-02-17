mod registry;
mod resources;
mod systems;
mod types;

pub use registry::ArtifactRegistry;
pub use resources::{Artifact, AvailableArtifacts, PlayerArtifacts, RerollCost, ShopOfferings};
pub use systems::{BuyRequest, RerollRequest, SellRequest};
pub use types::{ArtifactDef, ArtifactDefRaw, ArtifactId};

use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::wave::WavePhase;
use crate::GameState;
use systems::generate_shop_offerings;

pub struct ArtifactPlugin;

impl Plugin for ArtifactPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArtifactRegistry>()
            .init_resource::<PlayerArtifacts>()
            .init_resource::<ShopOfferings>()
            .init_resource::<AvailableArtifacts>()
            .init_resource::<RerollCost>()
            .add_systems(OnEnter(GameState::Playing), reset_artifacts)
            .add_message::<RerollRequest>()
            .add_message::<BuyRequest>()
            .add_message::<SellRequest>()
            .add_systems(OnEnter(WavePhase::ShopDelay), generate_shop_offerings)
            .add_systems(Update, (systems::handle_reroll, systems::handle_buy, systems::handle_sell));
    }
}

fn reset_artifacts(
    mut commands: Commands,
    mut artifacts: ResMut<PlayerArtifacts>,
    mut offerings: ResMut<ShopOfferings>,
    mut reroll_cost: ResMut<RerollCost>,
    balance: Res<GameBalance>,
) {
    artifacts.reset(&mut commands);
    artifacts.slots = vec![None; balance.shop.artifact_slots];
    offerings.clear(&mut commands);
    reroll_cost.reset_to(balance.shop.base_reroll_cost);
}
