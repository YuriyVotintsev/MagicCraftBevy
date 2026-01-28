use std::collections::HashMap;
use bevy::prelude::*;

use crate::abilities::ids::ParamId;
use crate::abilities::effect_def::ParamValue;
use crate::abilities::registry::ActivatorHandler;
use crate::abilities::{AbilityId, AbilityRegistry, ActivatorRegistry, EffectRegistry, AbilityContext};
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct IntervalActivations {
    pub entries: Vec<IntervalEntry>,
}

pub struct IntervalEntry {
    pub ability_id: AbilityId,
    pub interval: f32,
    pub timer: f32,
}

impl IntervalActivations {
    pub fn add(&mut self, ability_id: AbilityId, interval: f32) {
        self.entries.push(IntervalEntry {
            ability_id,
            interval,
            timer: interval,
        });
    }
}

pub fn interval_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut IntervalActivations,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut activations, stats, transform, faction) in &mut query {
        for entry in &mut activations.entries {
            entry.timer -= delta;

            if entry.timer > 0.0 {
                continue;
            }

            entry.timer = entry.interval;

            let Some(ability_def) = ability_registry.get(entry.ability_id) else {
                continue;
            };

            let ctx = AbilityContext::new(
                entity,
                *faction,
                stats,
                transform.translation,
                entry.ability_id,
            );

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }
        }
    }
}

#[derive(Default)]
pub struct IntervalHandler;

impl ActivatorHandler for IntervalHandler {
    fn name(&self) -> &'static str {
        "interval"
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        params: &HashMap<ParamId, ParamValue>,
        registry: &ActivatorRegistry,
    ) {
        let interval = extract_float_param(params, "interval", registry).unwrap_or(1.0);
        commands
            .entity(entity)
            .entry::<IntervalActivations>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id, interval));
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            interval_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn extract_float_param(
    params: &HashMap<ParamId, ParamValue>,
    name: &str,
    registry: &ActivatorRegistry,
) -> Option<f32> {
    let param_id = registry.get_param_id(name)?;
    match params.get(&param_id)? {
        ParamValue::Float(f) => Some(*f),
        _ => None,
    }
}

register_activator!(IntervalHandler);
