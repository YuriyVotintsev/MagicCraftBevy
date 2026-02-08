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

fn init_own_stats(
    commands: &mut Commands,
    entity_id: Entity,
    source: &AbilitySource,
    entity_def: &EntityDef,
    stat_registry: &StatRegistry,
    calculators: &StatCalculators,
) -> (ComputedStats, AbilitySource) {
    let mut modifiers = Modifiers::new();
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::new(stat_registry.len());

    dirty.mark_all((0..stat_registry.len() as u32).map(StatId));

    for (stat_id, value) in &entity_def.base_stats {
        modifiers.add(*stat_id, *value, None);
    }

    calculators.recalculate(&modifiers, &mut computed, &mut dirty);

    let self_source = AbilitySource {
        caster: TargetInfo::from_entity_and_position(
            entity_id,
            source.caster.position.unwrap_or(Vec2::ZERO),
        ),
        ..*source
    };

    commands.entity(entity_id).insert((
        self_source,
        source.caster_faction,
        modifiers,
        dirty,
        computed.clone(),
    ));

    (computed, self_source)
}

fn init_fsm(
    commands: &mut Commands,
    entity_id: Entity,
    states_block: &super::entity_def::StatesBlock,
    source: &AbilitySource,
    stats: &ComputedStats,
) -> Vec<ComponentDef> {
    let mut state_recalc_defs = Vec::new();

    if let Some(initial_state_def) = states_block.states.get(&states_block.initial) {
        let mut ec = commands.entity(entity_id);
        for comp in &initial_state_def.components {
            comp.insert_component(&mut ec, source, stats);
        }
        for trans in &initial_state_def.transitions {
            trans.insert_component(&mut ec, source, stats);
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

    state_recalc_defs
}

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

    let (effective_source, effective_stats) = if !entity_def.base_stats.is_empty() {
        let stat_registry = stat_registry.expect("base_stats requires StatRegistry");
        let calculators = calculators.expect("base_stats requires StatCalculators");
        let (owned_stats, self_source) =
            init_own_stats(commands, entity_id, source, entity_def, stat_registry, calculators);
        (self_source, owned_stats)
    } else {
        commands.entity(entity_id).insert((
            *source,
            source.caster_faction,
        ));
        (*source, ComputedStats::default())
    };

    let effective_stats_ref = if !entity_def.base_stats.is_empty() {
        &effective_stats
    } else {
        stats
    };

    let mut ec = commands.entity(entity_id);
    for component in &entity_def.components {
        component.insert_component(&mut ec, &effective_source, effective_stats_ref);
    }

    let recalc_defs: Vec<_> = entity_def.components.iter()
        .filter(|c| c.has_recalc())
        .cloned()
        .collect();

    if let Some(ability_registry) = ability_registry {
        for ability_name in &entity_def.abilities {
            if let Some(aid) = ability_registry.get_id(ability_name) {
                attach_ability(commands, entity_id, source.caster_faction, aid, ability_registry, false);
            }
        }
    }

    let state_recalc_defs = if let Some(states_block) = &entity_def.states {
        init_fsm(commands, entity_id, states_block, &effective_source, effective_stats_ref)
    } else {
        Vec::new()
    };

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
