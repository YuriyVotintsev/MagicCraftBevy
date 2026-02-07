use bevy::prelude::*;

use crate::stats::{ComputedStats, DirtyStats, Modifiers, StatCalculators, StatId, StatRegistry};
use super::components::ComponentDef;
use super::context::TargetInfo;
use super::entity_def::EntityDef;
use super::state::{CurrentState, StoredStatesBlock};
use super::{AbilitySource, AbilityRegistry, attach_ability};

#[derive(Component)]
pub struct StoredComponentDefs {
    pub base: Vec<ComponentDef>,
    pub state: Vec<ComponentDef>,
}

impl StoredComponentDefs {
    pub fn all(&self) -> impl Iterator<Item = &ComponentDef> {
        self.base.iter().chain(self.state.iter())
    }
}

#[allow(unused_assignments)]
pub fn spawn_entity(
    commands: &mut Commands,
    entity_def: &EntityDef,
    source: &AbilitySource,
    stats: &ComputedStats,
    stat_registry: Option<&StatRegistry>,
    calculators: Option<&StatCalculators>,
    ability_registry: Option<&AbilityRegistry>,
) -> Entity {
    let entity_id = commands.spawn_empty().id();

    let owned_stats: ComputedStats;
    let self_source;

    if !entity_def.base_stats.is_empty() {
        let base_stats = &entity_def.base_stats;
        let stat_registry = stat_registry.expect("base_stats requires StatRegistry");
        let calculators = calculators.expect("base_stats requires StatCalculators");

        let mut modifiers = Modifiers::new();
        let mut dirty = DirtyStats::default();
        let mut computed = ComputedStats::new(stat_registry.len());

        dirty.mark_all((0..stat_registry.len() as u32).map(StatId));

        for (stat_id, value) in base_stats {
            modifiers.add(*stat_id, *value, None);
        }

        calculators.recalculate(&modifiers, &mut computed, &mut dirty);

        let self_caster = TargetInfo::from_entity_and_position(
            entity_id,
            source.caster.position.unwrap_or(Vec2::ZERO),
        );

        commands.entity(entity_id).insert((
            AbilitySource {
                ability_id: source.ability_id,
                caster: self_caster,
                caster_faction: source.caster_faction,
                source: source.source,
                target: source.target,
                index: source.index,
                count: source.count,
            },
            source.caster_faction,
            modifiers,
            dirty,
        ));

        owned_stats = computed;
        commands.entity(entity_id).insert(owned_stats.clone());

        self_source = Some(AbilitySource {
            ability_id: source.ability_id,
            caster: self_caster,
            caster_faction: source.caster_faction,
            source: source.source,
            target: source.target,
            index: source.index,
            count: source.count,
        });
    } else {
        commands.entity(entity_id).insert((
            *source,
            source.caster_faction,
        ));

        owned_stats = ComputedStats::default();
        self_source = None;
    }

    let effective_source = self_source.as_ref().unwrap_or(source);
    let effective_stats = if self_source.is_some() { &owned_stats } else { stats };

    let mut ec = commands.entity(entity_id);
    for component in &entity_def.components {
        component.insert_component(&mut ec, effective_source, effective_stats);
    }

    let recalc_defs: Vec<_> = entity_def.components.iter()
        .filter(|c| c.has_recalc())
        .cloned()
        .collect();

    if !entity_def.abilities.is_empty() {
        if let Some(ability_registry) = ability_registry {
            for ability_name in &entity_def.abilities {
                if let Some(aid) = ability_registry.get_id(ability_name) {
                    attach_ability(commands, entity_id, source.caster_faction, aid, ability_registry, false);
                }
            }
        }
    }

    let mut state_recalc_defs = Vec::new();
    if let Some(states_block) = &entity_def.states {
        if let Some(initial_state_def) = states_block.states.get(&states_block.initial) {
            let mut ec = commands.entity(entity_id);
            for comp in &initial_state_def.components {
                comp.insert_component(&mut ec, effective_source, effective_stats);
            }
            for trans in &initial_state_def.transitions {
                trans.insert_component(&mut ec, effective_source, effective_stats);
            }

            state_recalc_defs.extend(
                initial_state_def.components.iter()
                    .chain(initial_state_def.transitions.iter())
                    .filter(|c| c.has_recalc())
                    .cloned()
            );
        }

        commands.entity(entity_id).insert((
            CurrentState(states_block.initial.clone()),
            StoredStatesBlock(states_block.clone()),
        ));
    }

    if !recalc_defs.is_empty() || !state_recalc_defs.is_empty() {
        commands.entity(entity_id).insert(StoredComponentDefs {
            base: recalc_defs,
            state: state_recalc_defs,
        });
    }

    entity_id
}

pub fn spawn_entity_def(
    commands: &mut Commands,
    entity_def: &EntityDef,
    source: &AbilitySource,
    stats: &ComputedStats,
    stat_registry: Option<&StatRegistry>,
    calculators: Option<&StatCalculators>,
    ability_registry: Option<&AbilityRegistry>,
) -> Vec<Entity> {
    let count = entity_def.count
        .as_ref()
        .map(|c| c.eval(source, stats).max(1.0) as usize)
        .unwrap_or(1);

    let mut entities = Vec::with_capacity(count);
    for i in 0..count {
        let iter_source = AbilitySource {
            index: i,
            count,
            ..*source
        };
        entities.push(spawn_entity(commands, entity_def, &iter_source, stats, stat_registry, calculators, ability_registry));
    }
    entities
}
