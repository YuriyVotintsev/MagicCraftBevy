use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;

use crate::GameState;
use crate::particles::{ParticleConfigRaw, ParticleRegistry};

#[derive(Resource, Default)]
pub struct LoadingState {
    pub particles_folder: Option<Handle<LoadedFolder>>,
}

pub fn start_loading(mut loading_state: ResMut<LoadingState>, asset_server: Res<AssetServer>) {
    info!("Starting asset loading...");
    loading_state.particles_folder = Some(asset_server.load_folder("particles"));
}

pub fn check_loaded(
    mut commands: Commands,
    loading_state: Res<LoadingState>,
    asset_server: Res<AssetServer>,
    particle_assets: Res<Assets<ParticleConfigRaw>>,
    folders: Res<Assets<LoadedFolder>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if !poll_folder(loading_state.particles_folder.as_ref(), &asset_server) {
        return;
    }

    let Some(folder_handle) = &loading_state.particles_folder else { return };
    resolve_particles(&mut commands, &asset_server, &particle_assets, &folders, folder_handle);

    #[cfg(feature = "dev")]
    if std::env::var("SKIP_MENU").is_ok() {
        info!("SKIP_MENU set, skipping main menu");
        next_game_state.set(GameState::Playing);
        return;
    }
    info!("Loading complete, transitioning to MainMenu");
    next_game_state.set(GameState::MainMenu);
}

fn poll_folder(handle: Option<&Handle<LoadedFolder>>, server: &AssetServer) -> bool {
    let Some(handle) = handle else { return false };
    matches!(server.get_load_state(handle.id()), Some(LoadState::Loaded))
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
        let Some(config) = particle_assets.get(typed_handle.id()) else { continue };
        let Some(path) = asset_server.get_path(typed_handle.id()) else { continue };
        let Some(name) = path.path().file_stem().and_then(|s| s.to_str()).and_then(|s| s.strip_suffix(".particle")) else { continue };
        info!("Loaded particle config: {}", name);
        raw_configs.insert(name.to_string(), config.clone());
    }
    commands.insert_resource(ParticleRegistry::resolve_all(&raw_configs));
}
