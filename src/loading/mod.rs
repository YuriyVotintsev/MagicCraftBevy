pub mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use assets::{
    AffixPoolAsset, AffixPoolLoader, ArtifactDefAsset, ArtifactDefLoader, BlueprintDefAsset,
    BlueprintDefLoader, GameBalanceAsset, GameBalanceLoader, HeroClassAsset, HeroClassLoader,
    OrbConfigAsset, OrbConfigLoader, StatsConfigAsset, StatsConfigLoader,
};
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StatsConfigAsset>()
            .init_asset::<BlueprintDefAsset>()
            .init_asset::<ArtifactDefAsset>()
            .init_asset::<HeroClassAsset>()
            .init_asset::<AffixPoolAsset>()
            .init_asset::<OrbConfigAsset>()
            .init_asset::<GameBalanceAsset>()
            .register_asset_loader(StatsConfigLoader)
            .register_asset_loader(BlueprintDefLoader)
            .register_asset_loader(ArtifactDefLoader)
            .register_asset_loader(HeroClassLoader)
            .register_asset_loader(AffixPoolLoader)
            .register_asset_loader(OrbConfigLoader)
            .register_asset_loader(GameBalanceLoader)
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
