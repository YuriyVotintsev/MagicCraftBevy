use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;

use crate::artifacts::{ArtifactDef, ArtifactRegistry, AvailableArtifacts};
use crate::blueprints::BlueprintRegistry;
use crate::player::hero_class::{AvailableHeroes, HeroClass, HeroClassModifier};
use crate::stats::{AggregationType, Expression, StatCalculators, StatId, StatRegistry};

use super::assets::{ArtifactDefAsset, BlueprintDefAsset, HeroClassAsset, StatsConfigAsset};

#[derive(Resource, Default)]
pub struct LoadingState {
    pub phase: LoadingPhase,
    pub stats_handle: Option<Handle<StatsConfigAsset>>,
    pub heroes_folder: Option<Handle<LoadedFolder>>,
    pub abilities_folder: Option<Handle<LoadedFolder>>,
    pub mobs_folder: Option<Handle<LoadedFolder>>,
    pub artifacts_folder: Option<Handle<LoadedFolder>>,
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
    loading_state.phase = LoadingPhase::WaitingForStats;
}

pub fn check_stats_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    stats_assets: Res<Assets<StatsConfigAsset>>,
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

    info!("Stats loaded, building registry...");

    let mut registry = StatRegistry::new();
    for def in &stats_config.stat_ids {
        registry.insert(&def.name, def.aggregation.clone());
    }

    let mut calculators = StatCalculators::new(registry.len());

    for def in &stats_config.stat_ids {
        let stat_id = registry.get(&def.name)
            .unwrap_or_else(|| panic!("Stat '{}' not found in registry", def.name));
        match &def.aggregation {
            AggregationType::Sum => {
                calculators.set(stat_id, Expression::ModifierSum(stat_id), vec![]);
            }
            AggregationType::Product => {
                calculators.set(stat_id, Expression::ModifierProduct(stat_id), vec![]);
            }
            AggregationType::Standard { base, increased, more } => {
                let base_id = registry.get(base)
                    .unwrap_or_else(|| panic!("Stat '{}' (base for '{}') not found in registry", base, def.name));
                let increased_id = registry.get(increased)
                    .unwrap_or_else(|| panic!("Stat '{}' (increased for '{}') not found in registry", increased, def.name));
                let more_id = registry.get(more)
                    .unwrap_or_else(|| panic!("Stat '{}' (more for '{}') not found in registry", more, def.name));

                let formula = Expression::Mul(
                    Box::new(Expression::Mul(
                        Box::new(Expression::Stat(base_id)),
                        Box::new(Expression::Add(
                            Box::new(Expression::Constant(1.0)),
                            Box::new(Expression::Stat(increased_id)),
                        )),
                    )),
                    Box::new(Expression::Stat(more_id)),
                );

                let depends_on = vec![base_id, increased_id, more_id];
                calculators.set(stat_id, formula, depends_on);
            }
            AggregationType::Custom => {}
        }
    }

    for calc in &stats_config.calculators {
        let stat_id = registry.get(&calc.stat)
            .unwrap_or_else(|| panic!("Calculator references unknown stat '{}'", calc.stat));
        let formula = calc.formula.resolve(&registry);
        let deps: Vec<StatId> = calc
            .depends_on
            .iter()
            .filter_map(|s| registry.get(s))
            .collect();
        calculators.set(stat_id, formula, deps);
    }

    calculators.rebuild();

    commands.insert_resource(registry);
    commands.insert_resource(calculators);

    loading_state.heroes_folder = Some(asset_server.load_folder("heroes"));
    loading_state.abilities_folder = Some(asset_server.load_folder("player_abilities"));
    loading_state.mobs_folder = Some(asset_server.load_folder("mobs"));
    loading_state.artifacts_folder = Some(asset_server.load_folder("artifacts"));

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
    folders: Res<Assets<LoadedFolder>>,
    stat_registry: Option<Res<StatRegistry>>,
    mut blueprint_registry: ResMut<BlueprintRegistry>,
    mut artifact_registry: ResMut<ArtifactRegistry>,
) {
    if loading_state.phase != LoadingPhase::WaitingForContent {
        return;
    }

    let Some(stat_registry) = stat_registry else {
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

    for handle in [
        heroes_folder_handle,
        abilities_folder_handle,
        mobs_folder_handle,
        artifacts_folder_handle,
    ] {
        if !matches!(
            asset_server.get_load_state(handle.id()),
            Some(LoadState::Loaded)
        ) {
            return;
        }
    }

    info!("All content loaded, finalizing...");

    let mut base_blueprint = None;
    let mut classes = Vec::new();
    if let Some(folder) = folders.get(heroes_folder_handle.id()) {
        for handle in &folder.handles {
            if let Ok(typed_bp) = handle.clone().try_typed::<BlueprintDefAsset>() {
                if let Some(blueprint_asset) = blueprint_assets.get(typed_bp.id()) {
                    let blueprint_def = blueprint_asset.0.resolve(&stat_registry);
                    info!("Registered base hero: {}", blueprint_asset.0.id);
                    let id = blueprint_registry.register(&blueprint_asset.0.id, blueprint_def);
                    base_blueprint = Some(id);
                }
            }

            if let Ok(typed_class) = handle.clone().try_typed::<HeroClassAsset>() {
                if let Some(class_asset) = hero_class_assets.get(typed_class.id()) {
                    let raw = &class_asset.0;
                    let modifiers: Vec<_> = raw.modifiers.iter()
                        .filter_map(|(name, &value)| {
                            stat_registry.get(name).map(|stat| HeroClassModifier {
                                stat,
                                value,
                                name: name.clone(),
                            })
                        })
                        .collect();
                    info!("Registered hero class: {}", raw.id);
                    classes.push(HeroClass {
                        id: raw.id.clone(),
                        display_name: raw.display_name.clone(),
                        color: raw.color,
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
                    let blueprint_def = blueprint_asset.0.resolve(&stat_registry);
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
                let def = ArtifactDef {
                    name: raw.name.clone(),
                    price: raw.price,
                };
                info!("Registered artifact: {}", raw.id);
                let id = artifact_registry.register(&raw.id, def);
                artifact_ids.push(id);
            }
        }
    }
    commands.insert_resource(AvailableArtifacts(artifact_ids));

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
