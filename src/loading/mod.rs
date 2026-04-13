pub mod assets;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use crate::actors::abilities::AbilitiesBalance;
use crate::actors::mobs::MobsBalance;
use crate::balance::GameBalance;
use crate::particles::ParticleConfigRaw;
use assets::RonAssetLoader;
use systems::LoadingState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        crate::palette::init();

        app.init_asset::<GameBalance>()
            .init_asset::<MobsBalance>()
            .init_asset::<AbilitiesBalance>()
            .init_asset::<ParticleConfigRaw>()
            .register_asset_loader(RonAssetLoader::<GameBalance>::default())
            .register_asset_loader(RonAssetLoader::<MobsBalance>::default())
            .register_asset_loader(RonAssetLoader::<AbilitiesBalance>::default())
            .register_asset_loader(RonAssetLoader::<ParticleConfigRaw>::default())
            .init_resource::<LoadingState>()
            .add_systems(OnEnter(GameState::Loading), systems::start_loading)
            .add_systems(
                Update,
                systems::check_loaded.run_if(in_state(GameState::Loading)),
            );
    }
}
