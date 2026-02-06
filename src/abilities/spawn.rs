use bevy::prelude::*;

use crate::Faction;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, StatCalculators, StatId, StatRegistry};
use super::context::TargetInfo;
use super::eval_context::EvalContext;
use super::entity_def::EntityDef;
use super::components::ComponentDef;
use super::state::{CurrentState, StoredStatesBlock};
use super::{AbilityInputs, AbilitySource, AbilityRegistry, attach_ability, ids::AbilityId};

#[derive(Component, Clone)]
pub struct StoredComponentDefs {
    pub defs: Vec<ComponentDef>,
}

pub struct SpawnContext<'a> {
    pub ability_id: AbilityId,
    pub caster: TargetInfo,
    pub caster_faction: Faction,
    pub source: TargetInfo,
    pub target: TargetInfo,
    pub stats: &'a ComputedStats,
    pub index: usize,
    pub count: usize,
}

impl<'a> SpawnContext<'a> {
    pub fn eval_context(&self) -> EvalContext<'a> {
        EvalContext {
            caster: self.caster,
            source: self.source,
            target: self.target,
            stats: self.stats,
            index: self.index,
            count: self.count,
        }
    }
}

#[allow(unused_assignments)]
pub fn spawn_entity(
    commands: &mut Commands,
    entity_def: &EntityDef,
    ctx: &SpawnContext,
    stat_registry: Option<&StatRegistry>,
    calculators: Option<&StatCalculators>,
    ability_registry: Option<&AbilityRegistry>,
) -> Entity {
    let entity_id = commands.spawn_empty().id();

    let owned_stats: ComputedStats;
    let self_ctx;

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
            ctx.caster.position.unwrap_or(Vec2::ZERO),
        );

        commands.entity(entity_id).insert((
            AbilitySource {
                ability_id: ctx.ability_id,
                caster: self_caster,
                caster_faction: ctx.caster_faction,
                source: ctx.source,
                target: ctx.target,
            },
            ctx.caster_faction,
            modifiers,
            dirty,
            AbilityInputs::new(),
        ));

        owned_stats = computed;
        commands.entity(entity_id).insert(owned_stats.clone());

        self_ctx = Some(SpawnContext {
            ability_id: ctx.ability_id,
            caster: self_caster,
            caster_faction: ctx.caster_faction,
            source: ctx.source,
            target: ctx.target,
            stats: &owned_stats,
            index: ctx.index,
            count: ctx.count,
        });
    } else {
        commands.entity(entity_id).insert((
            AbilitySource {
                ability_id: ctx.ability_id,
                caster: ctx.caster,
                caster_faction: ctx.caster_faction,
                source: ctx.source,
                target: ctx.target,
            },
            ctx.caster_faction,
        ));

        owned_stats = ComputedStats::default();
        self_ctx = None;
    }

    let effective_ctx = self_ctx.as_ref().unwrap_or(ctx);

    let has_recalculate = entity_def.components.iter().any(|c| matches!(c, ComponentDef::Recalculate(_)));

    let mut ec = commands.entity(entity_id);
    for component in &entity_def.components {
        component.insert_component(&mut ec, effective_ctx);
    }

    if has_recalculate {
        ec.insert(StoredComponentDefs {
            defs: entity_def.components.clone(),
        });
    }

    if !entity_def.abilities.is_empty() {
        if let Some(ability_registry) = ability_registry {
            for ability_name in &entity_def.abilities {
                if let Some(aid) = ability_registry.get_id(ability_name) {
                    attach_ability(commands, entity_id, ctx.caster_faction, aid, ability_registry);
                }
            }
        }
    }

    if let Some(states_block) = &entity_def.states {
        if let Some(initial_state_def) = states_block.states.get(&states_block.initial) {
            let mut ec = commands.entity(entity_id);
            for comp in &initial_state_def.components {
                comp.insert_component(&mut ec, effective_ctx);
            }
            for trans in &initial_state_def.transitions {
                trans.insert_component(&mut ec, effective_ctx);
            }
        }

        commands.entity(entity_id).insert((
            CurrentState(states_block.initial.clone()),
            StoredStatesBlock(states_block.clone()),
        ));
    }

    entity_id
}

pub fn spawn_entity_def(
    commands: &mut Commands,
    entity_def: &EntityDef,
    ctx: &SpawnContext,
    stat_registry: Option<&StatRegistry>,
    calculators: Option<&StatCalculators>,
    ability_registry: Option<&AbilityRegistry>,
) -> Vec<Entity> {
    let count = entity_def.count
        .as_ref()
        .map(|c| c.eval(&ctx.eval_context()).max(1.0) as usize)
        .unwrap_or(1);

    let mut entities = Vec::with_capacity(count);
    for i in 0..count {
        let spawn_ctx = SpawnContext {
            ability_id: ctx.ability_id,
            caster: ctx.caster,
            caster_faction: ctx.caster_faction,
            source: ctx.source,
            target: ctx.target,
            stats: ctx.stats,
            index: i,
            count,
        };
        entities.push(spawn_entity(commands, entity_def, &spawn_ctx, stat_registry, calculators, ability_registry));
    }
    entities
}
