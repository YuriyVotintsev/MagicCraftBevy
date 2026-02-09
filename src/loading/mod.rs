pub mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use assets::{
    ArtifactDefAsset, ArtifactDefLoader, BlueprintDefAsset, BlueprintDefLoader,
    HeroClassAsset, HeroClassLoader, StatsConfigAsset, StatsConfigLoader,
};
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StatsConfigAsset>()
            .init_asset::<BlueprintDefAsset>()
            .init_asset::<ArtifactDefAsset>()
            .init_asset::<HeroClassAsset>()
            .register_asset_loader(StatsConfigLoader)
            .register_asset_loader(BlueprintDefLoader)
            .register_asset_loader(ArtifactDefLoader)
            .register_asset_loader(HeroClassLoader)
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
