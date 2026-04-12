pub mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use assets::{
    GameBalanceAsset, GameBalanceLoader,
    StatsConfigAsset, StatsConfigLoader,
};
use crate::particles::{ParticleConfigRaw, ParticleConfigLoader};
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        crate::palette::init();

        app.init_asset::<StatsConfigAsset>()
            .init_asset::<GameBalanceAsset>()
            .init_asset::<ParticleConfigRaw>()
            .register_asset_loader(StatsConfigLoader)
            .register_asset_loader(GameBalanceLoader)
            .register_asset_loader(ParticleConfigLoader)
            .init_resource::<LoadingState>()
            .add_systems(OnEnter(GameState::Loading), systems::start_loading)
            .add_systems(
                Update,
                (
                    systems::check_stats_loaded,
                    systems::check_content_loaded,
                    systems::finalize_loading,
                )
                    .chain()
                    .run_if(in_state(GameState::Loading)),
            );
    }
}
