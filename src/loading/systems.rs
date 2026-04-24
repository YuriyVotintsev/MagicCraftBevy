use bevy::asset::LoadState;
#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::LoadedFolder;
use bevy::prelude::*;

use crate::GameState;
use crate::particles::{ParticleConfigRaw, ParticleRegistry};

#[cfg(target_arch = "wasm32")]
const PARTICLE_NAMES: &[&str] = &[
    "enemy_ability_death",
    "enemy_death",
    "enemy_death_large",
    "hit_burst",
    "player_death",
    "spinner_trail",
    "summon_grow",
    "tower_explosion",
];

#[derive(Resource, Default)]
pub struct LoadingState {
    #[cfg(not(target_arch = "wasm32"))]
    pub particles_folder: Option<Handle<LoadedFolder>>,
    #[cfg(target_arch = "wasm32")]
    pub particles: Vec<(String, Handle<ParticleConfigRaw>)>,
}

pub fn start_loading(mut loading_state: ResMut<LoadingState>, asset_server: Res<AssetServer>) {
    info!("Starting asset loading...");
    #[cfg(not(target_arch = "wasm32"))]
    {
        loading_state.particles_folder = Some(asset_server.load_folder("particles"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        loading_state.particles = PARTICLE_NAMES
            .iter()
            .map(|name| {
                let handle = asset_server
                    .load::<ParticleConfigRaw>(format!("particles/{name}.particle.ron"));
                ((*name).to_string(), handle)
            })
            .collect();
    }
}

pub fn check_loaded(
    mut commands: Commands,
    loading_state: Res<LoadingState>,
    asset_server: Res<AssetServer>,
    particle_assets: Res<Assets<ParticleConfigRaw>>,
    #[cfg(not(target_arch = "wasm32"))] folders: Res<Assets<LoadedFolder>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if !poll_folder(loading_state.particles_folder.as_ref(), &asset_server) {
            return;
        }

        let Some(folder_handle) = &loading_state.particles_folder else { return };
        resolve_particles_from_folder(
            &mut commands,
            &asset_server,
            &particle_assets,
            &folders,
            folder_handle,
        );
    }

    #[cfg(target_arch = "wasm32")]
    {
        if !poll_particles(&loading_state.particles, &asset_server) {
            return;
        }
        resolve_particles_from_list(&mut commands, &loading_state.particles, &particle_assets);
    }

    #[cfg(feature = "dev")]
    if std::env::var("SKIP_MENU").is_ok() {
        info!("SKIP_MENU set, skipping main menu");
        next_game_state.set(GameState::Playing);
        return;
    }
    info!("Loading complete, transitioning to MainMenu");
    next_game_state.set(GameState::MainMenu);
}

#[cfg(not(target_arch = "wasm32"))]
fn poll_folder(handle: Option<&Handle<LoadedFolder>>, server: &AssetServer) -> bool {
    let Some(handle) = handle else { return false };
    matches!(server.get_load_state(handle.id()), Some(LoadState::Loaded))
}

#[cfg(target_arch = "wasm32")]
fn poll_particles(
    particles: &[(String, Handle<ParticleConfigRaw>)],
    server: &AssetServer,
) -> bool {
    if particles.is_empty() {
        return false;
    }
    particles
        .iter()
        .all(|(_, h)| matches!(server.get_load_state(h.id()), Some(LoadState::Loaded)))
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_particles_from_folder(
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

#[cfg(target_arch = "wasm32")]
fn resolve_particles_from_list(
    commands: &mut Commands,
    particles: &[(String, Handle<ParticleConfigRaw>)],
    particle_assets: &Assets<ParticleConfigRaw>,
) {
    let mut raw_configs = std::collections::HashMap::new();
    for (name, handle) in particles {
        let Some(config) = particle_assets.get(handle.id()) else { continue };
        raw_configs.insert(name.clone(), config.clone());
    }
    commands.insert_resource(ParticleRegistry::resolve_all(&raw_configs));
}
