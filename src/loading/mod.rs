mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use crate::actors::MobsBalance;
use crate::balance::GameBalance;
use crate::particles::ParticleConfigRaw;
use crate::rune::RuneCosts;
use crate::wave::WavesConfig;
use assets::RonAssetLoader;
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        crate::palette::init();

        app.init_asset::<GameBalance>()
            .init_asset::<MobsBalance>()
            .init_asset::<ParticleConfigRaw>()
            .init_asset::<RuneCosts>()
            .init_asset::<WavesConfig>()
            .register_asset_loader(RonAssetLoader::<GameBalance>::default())
            .register_asset_loader(RonAssetLoader::<MobsBalance>::default())
            .register_asset_loader(RonAssetLoader::<ParticleConfigRaw>::default())
            .register_asset_loader(RonAssetLoader::<RuneCosts>::default())
            .register_asset_loader(RonAssetLoader::<WavesConfig>::default())
            .init_resource::<LoadingState>()
            .add_systems(OnEnter(GameState::Loading), systems::start_loading)
            .add_systems(
                Update,
                systems::check_loaded.run_if(in_state(GameState::Loading)),
            );
    }
}
