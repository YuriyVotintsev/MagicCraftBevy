use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::abilities::{
    AbilityDef, AbilityDefRaw, AbilityRegistry, ActivatorDef, ActivatorDefRaw, ActivatorRegistry,
    EffectDef, EffectDefRaw, EffectRegistry, ParamValue, ParamValueRaw,
};
use crate::fsm::MobRegistry;
use crate::player::PlayerDefResource;
use crate::stats::{AggregationType, Expression, StatCalculators, StatId, StatRegistry};

use super::assets::{AbilityDefAsset, MobDefAsset, PlayerDefAsset, StatsConfigAsset};

#[derive(Resource, Default)]
pub struct LoadingState {
    pub phase: LoadingPhase,
    pub stats_handle: Option<Handle<StatsConfigAsset>>,
    pub player_handle: Option<Handle<PlayerDefAsset>>,
    pub mob_handles: Vec<Handle<MobDefAsset>>,
    pub ability_handles: Vec<Handle<AbilityDefAsset>>,
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
        let stat_id = registry.get(&def.name).unwrap();
        match &def.aggregation {
            AggregationType::Sum => {
                calculators.set(stat_id, Expression::ModifierSum(stat_id), vec![]);
            }
            AggregationType::Product => {
                calculators.set(stat_id, Expression::ModifierProduct(stat_id), vec![]);
            }
            AggregationType::Standard { base, increased, more } => {
                let base_id = registry.get(base).unwrap();
                let increased_id = registry.get(increased).unwrap();
                let more_id = registry.get(more).unwrap();

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
        let stat_id = registry.get(&calc.stat).unwrap();
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

    loading_state.player_handle = Some(asset_server.load("player.player.ron"));

    loading_state.mob_handles = vec![
        asset_server.load("mobs/slime.mob.ron"),
        asset_server.load("mobs/archer.mob.ron"),
    ];

    loading_state.ability_handles = vec![
        asset_server.load("abilities/fireball.ability.ron"),
        asset_server.load("abilities/archer_shot.ability.ron"),
        asset_server.load("abilities/orbiting_orbs.ability.ron"),
    ];

    loading_state.phase = LoadingPhase::WaitingForContent;
    info!("Loading content assets...");
}

pub fn check_content_loaded(
    mut commands: Commands,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    player_assets: Res<Assets<PlayerDefAsset>>,
    mob_assets: Res<Assets<MobDefAsset>>,
    ability_assets: Res<Assets<AbilityDefAsset>>,
    stat_registry: Option<Res<StatRegistry>>,
    activator_registry: Res<ActivatorRegistry>,
    mut effect_registry: ResMut<EffectRegistry>,
    mut ability_registry: ResMut<AbilityRegistry>,
) {
    if loading_state.phase != LoadingPhase::WaitingForContent {
        return;
    }

    let Some(stat_registry) = stat_registry else {
        return;
    };

    let Some(player_handle) = &loading_state.player_handle else {
        return;
    };
    if !matches!(
        asset_server.get_load_state(player_handle.id()),
        Some(LoadState::Loaded)
    ) {
        return;
    }

    for handle in &loading_state.mob_handles {
        if !matches!(
            asset_server.get_load_state(handle.id()),
            Some(LoadState::Loaded)
        ) {
            return;
        }
    }

    for handle in &loading_state.ability_handles {
        if !matches!(
            asset_server.get_load_state(handle.id()),
            Some(LoadState::Loaded)
        ) {
            return;
        }
    }

    info!("All content loaded, finalizing...");

    let player_def = player_assets.get(player_handle.id()).unwrap();
    commands.insert_resource(PlayerDefResource(player_def.0.clone()));

    let mut mob_registry = MobRegistry::new();
    for handle in &loading_state.mob_handles {
        if let Some(mob_asset) = mob_assets.get(handle.id()) {
            info!("Registered mob: {}", mob_asset.0.name);
            mob_registry.insert(mob_asset.0.clone());
        }
    }
    commands.insert_resource(mob_registry);

    for handle in &loading_state.ability_handles {
        if let Some(ability_asset) = ability_assets.get(handle.id()) {
            let ability_def = resolve_ability_def(
                &ability_asset.0,
                &stat_registry,
                &mut ability_registry,
                &activator_registry,
                &mut effect_registry,
            );
            ability_registry.register(ability_def);
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

fn resolve_param_value(
    raw: &ParamValueRaw,
    stat_registry: &StatRegistry,
    effect_registry: &mut EffectRegistry,
) -> ParamValue {
    match raw {
        ParamValueRaw::Float(v) => ParamValue::Float(*v),
        ParamValueRaw::Int(v) => ParamValue::Int(*v),
        ParamValueRaw::Bool(v) => ParamValue::Bool(*v),
        ParamValueRaw::String(v) => ParamValue::String(v.clone()),
        ParamValueRaw::Stat(name) => {
            let stat_id = stat_registry.get(name).unwrap();
            ParamValue::Stat(stat_id)
        }
        ParamValueRaw::Effect(raw_effect) => {
            let effect = resolve_effect_def(raw_effect, stat_registry, effect_registry);
            ParamValue::Effect(Box::new(effect))
        }
        ParamValueRaw::EffectList(raw_effects) => {
            let effects = raw_effects
                .iter()
                .map(|e| resolve_effect_def(e, stat_registry, effect_registry))
                .collect();
            ParamValue::EffectList(effects)
        }
    }
}

fn resolve_effect_def(
    raw: &EffectDefRaw,
    stat_registry: &StatRegistry,
    effect_registry: &mut EffectRegistry,
) -> EffectDef {
    let effect_type = effect_registry.get_id(&raw.effect_type).unwrap();

    let params = raw
        .params
        .iter()
        .map(|(name, value)| {
            let param_id = effect_registry.get_or_insert_param_id(name);
            let resolved_value = resolve_param_value(value, stat_registry, effect_registry);
            (param_id, resolved_value)
        })
        .collect();

    EffectDef { effect_type, params }
}

fn resolve_activator_def(
    raw: &ActivatorDefRaw,
    stat_registry: &StatRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> ActivatorDef {
    let activator_type = activator_registry.get_id(&raw.activator_type).unwrap();

    let params = raw
        .params
        .iter()
        .map(|(name, value)| {
            let param_id = effect_registry.get_or_insert_param_id(name);
            let resolved_value = resolve_param_value(value, stat_registry, effect_registry);
            (param_id, resolved_value)
        })
        .collect();

    ActivatorDef { activator_type, params }
}

fn resolve_ability_def(
    raw: &AbilityDefRaw,
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> AbilityDef {
    use crate::abilities::ids::TagId;

    let id = ability_registry.allocate_id(&raw.id);

    let tags: Vec<TagId> = raw
        .tags
        .iter()
        .enumerate()
        .map(|(i, _)| TagId(i as u32))
        .collect();

    let activator = resolve_activator_def(&raw.activator, stat_registry, activator_registry, effect_registry);

    let effects = raw
        .effects
        .iter()
        .map(|e| resolve_effect_def(e, stat_registry, effect_registry))
        .collect();

    AbilityDef { id, tags, activator, effects }
}
