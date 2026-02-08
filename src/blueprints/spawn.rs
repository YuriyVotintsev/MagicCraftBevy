use bevy::ecs::query::QueryFilter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::Faction;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, StatCalculators, StatId, StatRegistry, DEFAULT_STATS};
use super::components::ComponentDef;
use super::context::TargetInfo;
use super::entity_def::{EntityDef, StatesBlock};
use super::recalc::StoredComponentDefs;
use super::state::{CurrentState, StoredStatesBlock};
use super::{SpawnSource, BlueprintRegistry, spawn_blueprint_entity};

#[derive(SystemParam)]
pub struct EntitySpawner<'w, 's> {
    pub commands: Commands<'w, 's>,
    stat_registry: Res<'w, StatRegistry>,
    calculators: Res<'w, StatCalculators>,
    blueprint_registry: Res<'w, BlueprintRegistry>,
}

impl EntitySpawner<'_, '_> {
    pub fn spawn(
        &mut self,
        entity_def: &EntityDef,
        parent_source: &SpawnSource,
        caster_stats: &ComputedStats,
    ) -> Vec<Entity> {
        let count = entity_def.count
            .as_ref()
            .map(|c| c.eval(parent_source, caster_stats).max(1.0) as usize)
            .unwrap_or(1);

        let mut entities = Vec::with_capacity(count);
        for i in 0..count {
            let iter_source = SpawnSource {
                index: i,
                count,
                ..*parent_source
            };
            entities.push(self.spawn_one(entity_def, &iter_source, caster_stats));
        }
        entities
    }

    pub fn spawn_triggered<F: QueryFilter>(
        &mut self,
        trigger_entity: Entity,
        source: &SpawnSource,
        source_info: TargetInfo,
        target_info: TargetInfo,
        entities: &[super::entity_def::EntityDef],
        stats_query: &Query<&ComputedStats>,
        transforms: &Query<&Transform, F>,
    ) {
        let caster_entity = source.caster.entity.unwrap_or(trigger_entity);
        let caster_stats = stats_query
            .get(caster_entity)
            .unwrap_or(&DEFAULT_STATS);
        let caster_pos = transforms
            .get(caster_entity)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let spawn_source = SpawnSource {
            blueprint_id: source.blueprint_id,
            caster: TargetInfo::from_entity_and_position(caster_entity, caster_pos),
            caster_faction: source.caster_faction,
            source: source_info,
            target: target_info,
            index: 0,
            count: 1,
        };

        for entity_def in entities {
            self.spawn(entity_def, &spawn_source, caster_stats);
        }
    }

    fn spawn_one(
        &mut self,
        entity_def: &EntityDef,
        parent_source: &SpawnSource,
        caster_stats: &ComputedStats,
    ) -> Entity {
        let entity_id = self.commands.spawn_empty().id();

        let (source, owned_stats) = self.init_identity(entity_id, parent_source, entity_def);
        let stats = owned_stats.as_ref().unwrap_or(caster_stats);

        let base_recalc = insert_components(
            &mut self.commands.entity(entity_id),
            &entity_def.components,
            &source,
            stats,
        );
        self.spawn_blueprint_entities(entity_id, &entity_def.abilities, source.caster_faction);
        let state_recalc = init_fsm(
            &mut self.commands,
            entity_id,
            entity_def.states.as_ref(),
            &source,
            stats,
        );
        store_recalc(&mut self.commands, entity_id, base_recalc, state_recalc);

        entity_id
    }

    fn init_identity(
        &mut self,
        entity_id: Entity,
        parent_source: &SpawnSource,
        entity_def: &EntityDef,
    ) -> (SpawnSource, Option<ComputedStats>) {
        if entity_def.base_stats.is_empty() {
            self.commands.entity(entity_id).insert((*parent_source, parent_source.caster_faction));
            return (*parent_source, None);
        }

        let mut modifiers = Modifiers::new();
        let mut dirty = DirtyStats::default();
        let mut computed = ComputedStats::new(self.stat_registry.len());

        dirty.mark_all((0..self.stat_registry.len() as u32).map(StatId));

        for (stat_id, value) in &entity_def.base_stats {
            modifiers.add(*stat_id, *value, None);
        }

        self.calculators.recalculate(&modifiers, &mut computed, &mut dirty);

        let source = SpawnSource {
            caster: TargetInfo::from_entity_and_position(
                entity_id,
                parent_source.caster.position.unwrap_or(Vec2::ZERO),
            ),
            ..*parent_source
        };

        self.commands.entity(entity_id).insert((
            source,
            parent_source.caster_faction,
            modifiers,
            dirty,
            computed.clone(),
        ));

        (source, Some(computed))
    }

    fn spawn_blueprint_entities(&mut self, entity_id: Entity, abilities: &[String], faction: Faction) {
        for name in abilities {
            if let Some(bid) = self.blueprint_registry.get_id(name) {
                spawn_blueprint_entity(&mut self.commands, entity_id, faction, bid, false);
            }
        }
    }
}

fn insert_components(
    ec: &mut EntityCommands,
    components: &[ComponentDef],
    source: &SpawnSource,
    stats: &ComputedStats,
) -> Vec<ComponentDef> {
    let mut recalc = Vec::new();
    for comp in components {
        comp.insert_component(ec, source, stats);
        if comp.has_recalc() {
            recalc.push(comp.clone());
        }
    }
    recalc
}

fn init_fsm(
    commands: &mut Commands,
    entity_id: Entity,
    states_block: Option<&StatesBlock>,
    source: &SpawnSource,
    stats: &ComputedStats,
) -> Vec<ComponentDef> {
    let Some(states_block) = states_block else { return Vec::new() };

    let mut recalc = Vec::new();
    if let Some(state_def) = states_block.states.get(states_block.initial) {
        let mut ec = commands.entity(entity_id);
        for comp in state_def.components.iter().chain(state_def.transitions.iter()) {
            comp.insert_component(&mut ec, source, stats);
            if comp.has_recalc() {
                recalc.push(comp.clone());
            }
        }
    }

    commands.entity(entity_id).insert((
        CurrentState(states_block.initial),
        StoredStatesBlock(states_block.clone()),
    ));

    recalc
}

fn store_recalc(
    commands: &mut Commands,
    entity_id: Entity,
    base: Vec<ComponentDef>,
    state: Vec<ComponentDef>,
) {
    if !base.is_empty() || !state.is_empty() {
        commands.entity(entity_id).insert(StoredComponentDefs { base, state });
    }
}
