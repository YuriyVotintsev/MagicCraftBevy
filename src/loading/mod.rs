pub mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use assets::{
    BlueprintDefAsset, BlueprintDefLoader, GameBalanceAsset, GameBalanceLoader,
    SkillTreeDefAsset, SkillTreeDefLoader, StatsConfigAsset, StatsConfigLoader,
};
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        crate::palette::init();

        app.init_asset::<StatsConfigAsset>()
            .init_asset::<BlueprintDefAsset>()
            .init_asset::<GameBalanceAsset>()
            .init_asset::<SkillTreeDefAsset>()
            .register_asset_loader(StatsConfigLoader)
            .register_asset_loader(BlueprintDefLoader)
            .register_asset_loader(GameBalanceLoader)
            .register_asset_loader(SkillTreeDefLoader)
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
