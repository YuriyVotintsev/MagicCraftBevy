pub mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use assets::{
    BlueprintDefAsset, BlueprintDefLoader, PlayerDefAsset,
    PlayerDefLoader, StatsConfigAsset, StatsConfigLoader,
};
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StatsConfigAsset>()
            .init_asset::<PlayerDefAsset>()
            .init_asset::<BlueprintDefAsset>()
            .register_asset_loader(StatsConfigLoader)
            .register_asset_loader(PlayerDefLoader)
            .register_asset_loader(BlueprintDefLoader)
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
