use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;

use crate::stats::display::StatDisplayRegistry;
use crate::expr::calc::CalcRegistry;
use crate::expr::parser::{TypedExpr, StatAtomParser, parse_expr_string_with};
use crate::stats::{StatEvalKind, StatCalculators, StatRegistry};
use crate::stats::loader::StatEvalKindRaw;

use crate::particles::{ParticleConfigRaw, ParticleRegistry};

use super::assets::{
    AbilitiesBalanceAsset, GameBalanceAsset,
    MobsBalanceAsset, StatsConfigAsset,
};

#[derive(Resource, Default)]
pub struct LoadingState {
    pub phase: LoadingPhase,
    pub stats_handle: Option<Handle<StatsConfigAsset>>,
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
    loading_state.stats_handle = Some(asset_server.load("stats/config.stats.ron"));
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
    stats_assets: Res<Assets<StatsConfigAsset>>,
    balance_assets: Res<Assets<GameBalanceAsset>>,
    mobs_balance_assets: Res<Assets<MobsBalanceAsset>>,
    abilities_balance_assets: Res<Assets<AbilitiesBalanceAsset>>,
) {
    if loading_state.phase != LoadingPhase::WaitingForStats {
        return;
    }

    let Some(handle) = &loading_state.stats_handle else { return };
    match asset_server.get_load_state(handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => { error!("Failed to load stats config: {:?}", err); return; }
        _ => return,
    }
    let Some(stats_config) = stats_assets.get(handle.id()) else { return };

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

    info!("Stats loaded, building registry...");

    let calc_registry = CalcRegistry::from_raw(&stats_config.calcs);
    let mut registry = StatRegistry::new();
    for def in &stats_config.stat_ids {
        registry.insert(&def.name, def.lower_is_better);
    }

    let mut calculators = StatCalculators::new(registry.len());
    for def in &stats_config.stat_ids {
        let stat_id = registry.get(&def.name).unwrap_or_else(|| panic!("Stat '{}' not found in registry", def.name));
        match &def.eval {
            StatEvalKindRaw::Sum => { calculators.set(stat_id, StatEvalKind::Sum, vec![]); }
            StatEvalKindRaw::Product => { calculators.set(stat_id, StatEvalKind::Product, vec![]); }
            StatEvalKindRaw::Formula(formula_str) => {
                let expanded = calc_registry.expand(formula_str).unwrap_or_else(|e| panic!("Failed to expand calc in stat '{}': {}", def.name, e));
                let parsed = match parse_expr_string_with(&expanded, &StatAtomParser) {
                    Ok(TypedExpr::Scalar(expr)) => expr,
                    Ok(_) => panic!("Stat '{}' formula must be a scalar expression", def.name),
                    Err(e) => panic!("Failed to parse formula for stat '{}': {}\nFormula: {}", def.name, e, formula_str),
                };
                let resolved = parsed.resolve(&|name: &str| registry.get(name));
                let mut deps = Vec::new();
                resolved.collect_stat_deps(&mut deps);
                deps.dedup();
                calculators.set(stat_id, StatEvalKind::Formula(resolved), deps);
            }
        }
    }
    calculators.rebuild();

    let display_registry = StatDisplayRegistry::new(stats_config.display.clone(), &registry);
    commands.insert_resource(display_registry);
    commands.insert_resource(registry);
    commands.insert_resource(calculators);
    commands.insert_resource(calc_registry);

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
