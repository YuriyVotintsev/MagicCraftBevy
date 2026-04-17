use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::GameState;
use crate::actors::MobsBalance;
use crate::balance::GameBalance;
use crate::particles::{ParticleConfigRaw, ParticleRegistry};
use crate::rune::{IconAssets, RuneCosts};
use crate::stats::Stat;

const ICON_SOURCES: &[(Stat, &str)] = &[
    (Stat::PhysicalDamage, "images/Icons/Damage.png"),
    (Stat::MaxLife, "images/Icons/Life.png"),
    (Stat::AttackSpeed, "images/Icons/AttackSpeed.png"),
];

#[derive(Resource, Default)]
pub struct LoadingState {
    pub balance_handle: Option<Handle<GameBalance>>,
    pub mobs_balance_handle: Option<Handle<MobsBalance>>,
    pub rune_costs_handle: Option<Handle<RuneCosts>>,
    pub particles_folder: Option<Handle<LoadedFolder>>,
    pub icons: Vec<(Stat, Handle<Image>)>,
    pub icons_processed: bool,
}

pub fn start_loading(mut loading_state: ResMut<LoadingState>, asset_server: Res<AssetServer>) {
    info!("Starting asset loading...");
    loading_state.balance_handle = Some(asset_server.load("balance.ron"));
    loading_state.mobs_balance_handle = Some(asset_server.load("mobs.ron"));
    loading_state.rune_costs_handle = Some(asset_server.load("runes.ron"));
    loading_state.particles_folder = Some(asset_server.load_folder("particles"));
    loading_state.icons = ICON_SOURCES
        .iter()
        .map(|(stat, path)| (*stat, asset_server.load::<Image>(*path)))
        .collect();
}

pub fn check_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    balance_assets: Res<Assets<GameBalance>>,
    mobs_balance_assets: Res<Assets<MobsBalance>>,
    rune_costs_assets: Res<Assets<RuneCosts>>,
    particle_assets: Res<Assets<ParticleConfigRaw>>,
    folders: Res<Assets<LoadedFolder>>,
    mut images: ResMut<Assets<Image>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let ready = poll_asset(loading_state.balance_handle.as_ref(), &asset_server, &balance_assets, &mut commands, "balance")
        && poll_asset(loading_state.mobs_balance_handle.as_ref(), &asset_server, &mobs_balance_assets, &mut commands, "mobs balance")
        && poll_asset(loading_state.rune_costs_handle.as_ref(), &asset_server, &rune_costs_assets, &mut commands, "rune costs")
        && poll_folder(loading_state.particles_folder.as_ref(), &asset_server)
        && poll_icons(&loading_state.icons, &asset_server);
    if !ready { return }

    let Some(folder_handle) = &loading_state.particles_folder else { return };
    resolve_particles(&mut commands, &asset_server, &particle_assets, &folders, folder_handle);

    if !loading_state.icons_processed {
        resolve_icons(&mut commands, &mut images, &loading_state.icons);
        loading_state.icons_processed = true;
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

fn poll_asset<A: Asset + Resource + Clone>(
    handle: Option<&Handle<A>>,
    server: &AssetServer,
    assets: &Assets<A>,
    commands: &mut Commands,
    label: &str,
) -> bool {
    let Some(handle) = handle else { return false };
    match server.get_load_state(handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => {
            error!("Failed to load {}: {:?}", label, err);
            return false;
        }
        _ => return false,
    }
    let Some(asset) = assets.get(handle.id()) else { return false };
    commands.insert_resource(asset.clone());
    true
}

fn poll_folder(handle: Option<&Handle<LoadedFolder>>, server: &AssetServer) -> bool {
    let Some(handle) = handle else { return false };
    matches!(server.get_load_state(handle.id()), Some(LoadState::Loaded))
}

fn poll_icons(icons: &[(Stat, Handle<Image>)], server: &AssetServer) -> bool {
    icons.iter().all(|(_, h)| {
        matches!(server.get_load_state(h.id()), Some(LoadState::Loaded))
    })
}

fn resolve_icons(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    icons: &[(Stat, Handle<Image>)],
) {
    let mut by_stat: HashMap<Stat, Handle<Image>> = HashMap::new();
    for (stat, handle) in icons {
        if let Some(image) = images.get_mut(handle.id()) {
            keyout_black_to_alpha(image);
        }
        by_stat.insert(*stat, handle.clone());
    }
    commands.insert_resource(IconAssets::new(by_stat));
}

fn keyout_black_to_alpha(image: &mut Image) {
    let Some(data) = image.data.as_mut() else { return };
    for chunk in data.chunks_exact_mut(4) {
        let alpha = chunk[0].max(chunk[1]).max(chunk[2]);
        chunk[0] = 255;
        chunk[1] = 255;
        chunk[2] = 255;
        chunk[3] = alpha;
    }
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
