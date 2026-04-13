use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;

use crate::particles::{ParticleConfigRaw, ParticleRegistry};

use super::assets::{
    AbilitiesBalanceAsset, GameBalanceAsset, MobsBalanceAsset,
};

#[derive(Resource, Default)]
pub struct LoadingState {
    pub phase: LoadingPhase,
    pub balance_handle: Option<Handle<GameBalanceAsset>>,
    pub mobs_balance_handle: Option<Handle<MobsBalanceAsset>>,
    pub abilities_balance_handle: Option<Handle<AbilitiesBalanceAsset>>,
    pub particles_folder: Option<Handle<LoadedFolder>>,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum LoadingPhase {
    #[default]
    StartingStatsLoad,
    WaitingForStats,
    WaitingForContent,
    Finalizing,
    Done,
}

pub fn start_loading(mut loading_state: ResMut<LoadingState>, asset_server: Res<AssetServer>) {
    info!("Starting asset loading...");
    loading_state.balance_handle = Some(asset_server.load("balance.ron"));
    loading_state.mobs_balance_handle = Some(asset_server.load("mobs.ron"));
    loading_state.abilities_balance_handle = Some(asset_server.load("abilities.ron"));
    loading_state.phase = LoadingPhase::WaitingForStats;
}

#[allow(clippy::too_many_arguments)]
pub fn check_stats_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    balance_assets: Res<Assets<GameBalanceAsset>>,
    mobs_balance_assets: Res<Assets<MobsBalanceAsset>>,
    abilities_balance_assets: Res<Assets<AbilitiesBalanceAsset>>,
) {
    if loading_state.phase != LoadingPhase::WaitingForStats {
        return;
    }

    let Some(balance_handle) = &loading_state.balance_handle else { return };
    match asset_server.get_load_state(balance_handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => { error!("Failed to load balance config: {:?}", err); return; }
        _ => return,
    }
    let Some(balance_asset) = balance_assets.get(balance_handle.id()) else { return };
    commands.insert_resource(balance_asset.0.clone());

    let Some(mobs_balance_handle) = &loading_state.mobs_balance_handle else { return };
    match asset_server.get_load_state(mobs_balance_handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => { error!("Failed to load mobs balance: {:?}", err); return; }
        _ => return,
    }
    let Some(mobs_balance_asset) = mobs_balance_assets.get(mobs_balance_handle.id()) else { return };
    commands.insert_resource(mobs_balance_asset.0.clone());

    let Some(abilities_balance_handle) = &loading_state.abilities_balance_handle else { return };
    match asset_server.get_load_state(abilities_balance_handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => { error!("Failed to load abilities balance: {:?}", err); return; }
        _ => return,
    }
    let Some(abilities_balance_asset) = abilities_balance_assets.get(abilities_balance_handle.id()) else { return };
    commands.insert_resource(abilities_balance_asset.0.clone());

    info!("Balance loaded, building stat system...");

    let (calculators, display_registry) = crate::stats::build_stat_system();
    commands.insert_resource(calculators);
    commands.insert_resource(display_registry);

    loading_state.particles_folder = Some(asset_server.load_folder("particles"));
    loading_state.phase = LoadingPhase::WaitingForContent;
    info!("Loading content assets...");
}

pub fn check_content_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    particle_assets: Res<Assets<ParticleConfigRaw>>,
    folders: Res<Assets<LoadedFolder>>,
) {
    if loading_state.phase != LoadingPhase::WaitingForContent { return }

    let Some(particles_folder_handle) = &loading_state.particles_folder else { return };
    if !matches!(asset_server.get_load_state(particles_folder_handle.id()), Some(LoadState::Loaded)) {
        return;
    }

    info!("All content loaded, finalizing...");
    resolve_particles(&mut commands, &asset_server, &particle_assets, &folders, particles_folder_handle);
    loading_state.phase = LoadingPhase::Finalizing;
}

pub fn finalize_loading(
    mut loading_state: ResMut<LoadingState>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    if loading_state.phase != LoadingPhase::Finalizing { return }
    loading_state.phase = LoadingPhase::Done;

    #[cfg(feature = "dev")]
    if std::env::var("SKIP_MENU").is_ok() {
        info!("SKIP_MENU set, skipping main menu");
        next_state.set(crate::GameState::Playing);
        return;
    }
    info!("Loading complete, transitioning to MainMenu");
    next_state.set(crate::GameState::MainMenu);
}

fn resolve_particles(
    commands: &mut Commands,
    asset_server: &AssetServer,
    particle_assets: &Assets<ParticleConfigRaw>,
    folders: &Assets<LoadedFolder>,
    folder_handle: &Handle<LoadedFolder>,
) {
    let Some(folder) = folders.get(folder_handle.id()) else { return };
    let mut raw_configs = std::collections::HashMap::new();
    for handle in &folder.handles {
        let typed_handle: Handle<ParticleConfigRaw> = handle.clone().typed();
        if let Some(config) = particle_assets.get(typed_handle.id()) {
            if let Some(path) = asset_server.get_path(typed_handle.id()) {
                let name = path.path().file_stem().and_then(|s| s.to_str()).unwrap_or("")
                    .strip_suffix(".particle").unwrap_or(path.path().file_stem().and_then(|s| s.to_str()).unwrap_or(""))
                    .to_string();
                if !name.is_empty() {
                    info!("Loaded particle config: {}", name);
                    raw_configs.insert(name, config.clone());
                }
            }
        }
    }
    let particle_registry = ParticleRegistry::resolve_all(&raw_configs);
    commands.insert_resource(particle_registry);
}
