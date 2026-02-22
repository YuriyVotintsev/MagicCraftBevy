use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;

use crate::affixes::{AffixDef, AffixRegistry, OrbDef, OrbRegistry};
use crate::artifacts::{ArtifactRegistry, AvailableArtifacts};
use crate::blueprints::BlueprintRegistry;
use crate::player::hero_class::{AvailableHeroes, HeroClass};
use crate::player::SpellSlot;
use crate::stats::display::StatDisplayRegistry;
use crate::expr::calc::CalcRegistry;
use crate::expr::parser::{TypedExpr, StatAtomParser, parse_expr_string_with};
use crate::stats::{StatEvalKind, StatCalculators, StatRegistry};
use crate::stats::loader::StatEvalKindRaw;

use super::assets::{
    AffixPoolAsset, ArtifactDefAsset, BlueprintDefAsset, GameBalanceAsset, HeroClassAsset,
    OrbConfigAsset, StatsConfigAsset,
};

#[derive(Resource, Default)]
pub struct LoadingState {
    pub phase: LoadingPhase,
    pub stats_handle: Option<Handle<StatsConfigAsset>>,
    pub balance_handle: Option<Handle<GameBalanceAsset>>,
    pub heroes_folder: Option<Handle<LoadedFolder>>,
    pub abilities_folder: Option<Handle<LoadedFolder>>,
    pub mobs_folder: Option<Handle<LoadedFolder>>,
    pub artifacts_folder: Option<Handle<LoadedFolder>>,
    pub affixes_folder: Option<Handle<LoadedFolder>>,
    pub orbs_handle: Option<Handle<OrbConfigAsset>>,
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
    loading_state.phase = LoadingPhase::WaitingForStats;
}

pub fn check_stats_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    stats_assets: Res<Assets<StatsConfigAsset>>,
    balance_assets: Res<Assets<GameBalanceAsset>>,
) {
    if loading_state.phase != LoadingPhase::WaitingForStats {
        return;
    }

    let Some(handle) = &loading_state.stats_handle else {
        return;
    };

    match asset_server.get_load_state(handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => {
            error!("Failed to load stats config: {:?}", err);
            return;
        }
        _ => return,
    }

    let Some(stats_config) = stats_assets.get(handle.id()) else {
        return;
    };

    let Some(balance_handle) = &loading_state.balance_handle else {
        return;
    };
    match asset_server.get_load_state(balance_handle.id()) {
        Some(LoadState::Loaded) => {}
        Some(LoadState::Failed(err)) => {
            error!("Failed to load balance config: {:?}", err);
            return;
        }
        _ => return,
    }
    let Some(balance_asset) = balance_assets.get(balance_handle.id()) else {
        return;
    };
    commands.insert_resource(balance_asset.0.clone());

    info!("Stats loaded, building registry...");

    let calc_registry = CalcRegistry::from_raw(&stats_config.calcs);

    let mut registry = StatRegistry::new();
    for def in &stats_config.stat_ids {
        registry.insert(&def.name, def.lower_is_better);
    }

    let mut calculators = StatCalculators::new(registry.len());

    for def in &stats_config.stat_ids {
        let stat_id = registry.get(&def.name)
            .unwrap_or_else(|| panic!("Stat '{}' not found in registry", def.name));
        match &def.eval {
            StatEvalKindRaw::Sum => {
                calculators.set(stat_id, StatEvalKind::Sum, vec![]);
            }
            StatEvalKindRaw::Product => {
                calculators.set(stat_id, StatEvalKind::Product, vec![]);
            }
            StatEvalKindRaw::Formula(formula_str) => {
                let expanded = calc_registry.expand(formula_str)
                    .unwrap_or_else(|e| panic!("Failed to expand calc in stat '{}': {}", def.name, e));
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

    loading_state.heroes_folder = Some(asset_server.load_folder("heroes"));
    loading_state.abilities_folder = Some(asset_server.load_folder("player_abilities"));
    loading_state.mobs_folder = Some(asset_server.load_folder("mobs"));
    loading_state.artifacts_folder = Some(asset_server.load_folder("artifacts"));
    loading_state.affixes_folder = Some(asset_server.load_folder("affixes"));
    loading_state.orbs_handle = Some(asset_server.load("orbs/config.orbs.ron"));

    loading_state.phase = LoadingPhase::WaitingForContent;
    info!("Loading content assets...");
}

pub fn check_content_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    blueprint_assets: Res<Assets<BlueprintDefAsset>>,
    hero_class_assets: Res<Assets<HeroClassAsset>>,
    artifact_assets: Res<Assets<ArtifactDefAsset>>,
    affix_pool_assets: Res<Assets<AffixPoolAsset>>,
    orb_config_assets: Res<Assets<OrbConfigAsset>>,
    folders: Res<Assets<LoadedFolder>>,
    stat_registry: Option<Res<StatRegistry>>,
    calc_registry: Option<Res<CalcRegistry>>,
    mut blueprint_registry: ResMut<BlueprintRegistry>,
    mut artifact_registry: ResMut<ArtifactRegistry>,
    mut affix_registry: ResMut<AffixRegistry>,
    mut orb_registry: ResMut<OrbRegistry>,
) {
    if loading_state.phase != LoadingPhase::WaitingForContent {
        return;
    }

    let Some(stat_registry) = stat_registry else {
        return;
    };
    let Some(calc_registry) = calc_registry else {
        return;
    };

    let Some(heroes_folder_handle) = &loading_state.heroes_folder else {
        return;
    };
    let Some(abilities_folder_handle) = &loading_state.abilities_folder else {
        return;
    };
    let Some(mobs_folder_handle) = &loading_state.mobs_folder else {
        return;
    };
    let Some(artifacts_folder_handle) = &loading_state.artifacts_folder else {
        return;
    };
    let Some(affixes_folder_handle) = &loading_state.affixes_folder else {
        return;
    };
    let Some(orbs_handle) = &loading_state.orbs_handle else {
        return;
    };

    for handle in [
        heroes_folder_handle,
        abilities_folder_handle,
        mobs_folder_handle,
        artifacts_folder_handle,
        affixes_folder_handle,
    ] {
        if !matches!(
            asset_server.get_load_state(handle.id()),
            Some(LoadState::Loaded)
        ) {
            return;
        }
    }

    if !matches!(
        asset_server.get_load_state(orbs_handle.id()),
        Some(LoadState::Loaded)
    ) {
        return;
    }

    info!("All content loaded, finalizing...");

    let lookup = |name: &str| stat_registry.get(name);

    let mut base_blueprint = None;
    let mut classes = Vec::new();
    if let Some(folder) = folders.get(heroes_folder_handle.id()) {
        for handle in &folder.handles {
            if let Ok(typed_bp) = handle.clone().try_typed::<BlueprintDefAsset>() {
                if let Some(blueprint_asset) = blueprint_assets.get(typed_bp.id()) {
                    let blueprint_def = blueprint_asset.0.resolve(&lookup, &calc_registry);
                    info!("Registered base hero: {}", blueprint_asset.0.id);
                    let id = blueprint_registry.register(&blueprint_asset.0.id, blueprint_def);
                    base_blueprint = Some(id);
                }
            }

            if let Ok(typed_class) = handle.clone().try_typed::<HeroClassAsset>() {
                if let Some(class_asset) = hero_class_assets.get(typed_class.id()) {
                    let raw = &class_asset.0;
                    let modifiers = raw.modifiers.iter()
                        .map(|m| m.resolve(&stat_registry))
                        .collect();
                    info!("Registered hero class: {}", raw.id);
                    classes.push(HeroClass {
                        display_name: raw.display_name.clone(),
                        color: raw.color,
                        sprite: raw.sprite.clone(),
                        modifiers,
                    });
                }
            }
        }
    }
    let base_blueprint = base_blueprint.expect("No base hero blueprint found in heroes folder");
    commands.insert_resource(AvailableHeroes { base_blueprint, classes });

    let folder_handles = [abilities_folder_handle.id(), mobs_folder_handle.id()];
    for folder_id in folder_handles {
        if let Some(folder) = folders.get(folder_id) {
            for handle in &folder.handles {
                let typed_handle: Handle<BlueprintDefAsset> = handle.clone().typed();
                if let Some(blueprint_asset) = blueprint_assets.get(typed_handle.id()) {
                    let blueprint_def = blueprint_asset.0.resolve(&lookup, &calc_registry);
                    info!("Registered blueprint: {}", blueprint_asset.0.id);
                    blueprint_registry.register(&blueprint_asset.0.id, blueprint_def);
                }
            }
        }
    }

    let mut artifact_ids = Vec::new();
    if let Some(folder) = folders.get(artifacts_folder_handle.id()) {
        for handle in &folder.handles {
            let typed_handle: Handle<ArtifactDefAsset> = handle.clone().typed();
            if let Some(artifact_asset) = artifact_assets.get(typed_handle.id()) {
                let raw = &artifact_asset.0;
                let def = raw.resolve(&stat_registry);
                info!("Registered artifact: {}", raw.id);
                let id = artifact_registry.register(&raw.id, def);
                artifact_ids.push(id);
            }
        }
    }
    commands.insert_resource(AvailableArtifacts::new(artifact_ids));

    if let Some(folder) = folders.get(affixes_folder_handle.id()) {
        for handle in &folder.handles {
            let typed_handle: Handle<AffixPoolAsset> = handle.clone().typed();
            if let Some(pool_asset) = affix_pool_assets.get(typed_handle.id()) {
                let path = asset_server
                    .get_path(handle.id())
                    .map(|p| p.path().to_string_lossy().to_string())
                    .unwrap_or_default();
                let slot = if path.contains("active") {
                    SpellSlot::Active
                } else if path.contains("passive") {
                    SpellSlot::Passive
                } else if path.contains("defensive") {
                    SpellSlot::Defensive
                } else {
                    warn!("Unknown affix pool file: {}", path);
                    continue;
                };
                for raw in &pool_asset.0 {
                    let tiers = raw.tiers.iter()
                        .map(|t| t.resolve(&stat_registry))
                        .collect();
                    let def = AffixDef { tiers };
                    let id = affix_registry.register(def, slot);
                    info!("Registered affix: {} ({:?}) for {:?}", raw.id, id, slot);
                }
            }
        }
    }

    if let Some(orb_asset) = orb_config_assets.get(orbs_handle.id()) {
        for raw in &orb_asset.0 {
            let def = OrbDef {
                name: raw.name.clone(),
                description: raw.description.clone(),
                price: raw.price,
                behavior: raw.orb_type,
            };
            let id = orb_registry.register(&raw.id, def);
            info!("Registered orb: {} ({:?})", raw.name, id);
        }
    }

    loading_state.phase = LoadingPhase::Finalizing;
}

pub fn finalize_loading(
    mut loading_state: ResMut<LoadingState>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    if loading_state.phase != LoadingPhase::Finalizing {
        return;
    }

    info!("Loading complete, transitioning to MainMenu");
    loading_state.phase = LoadingPhase::Done;
    next_state.set(crate::GameState::MainMenu);
}
