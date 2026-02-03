use bevy::prelude::*;

use crate::Faction;
use crate::stats::ComputedStats;
use super::ids::AbilityId;
use super::context::Target;
use super::entity_def::EntityDef;
use super::AbilitySource;

pub struct SpawnContext<'a> {
    pub ability_id: AbilityId,
    pub caster: Entity,
    pub caster_faction: Faction,
    pub source: Target,
    pub target: Option<Target>,
    pub stats: &'a ComputedStats,
    pub index: usize,
    pub count: usize,
}

#[derive(Component, Clone)]
pub struct AbilityContextComponent {
    pub source: Target,
    pub target: Option<Target>,
}

pub fn spawn_entity(commands: &mut Commands, entity_def: &EntityDef, ctx: &SpawnContext) -> Entity {
    let mut ec = commands.spawn((
        AbilitySource::new(ctx.ability_id, ctx.caster, ctx.caster_faction),
        AbilityContextComponent {
            source: ctx.source,
            target: ctx.target,
        },
        ctx.caster_faction,
    ));

    for component in &entity_def.components {
        component.spawn(&mut ec, ctx);
    }

    ec.id()
}

pub fn spawn_entity_def(commands: &mut Commands, entity_def: &EntityDef, ctx: &SpawnContext) -> Vec<Entity> {
    let count = entity_def.count
        .as_ref()
        .map(|c| c.evaluate_i32(ctx.stats).max(1) as usize)
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
        entities.push(spawn_entity(commands, entity_def, &spawn_ctx));
    }
    entities
}

pub fn spawn_entities(commands: &mut Commands, entity_defs: &[EntityDef], ctx: &SpawnContext) -> Vec<Entity> {
    entity_defs.iter().flat_map(|def| spawn_entity_def(commands, def, ctx)).collect()
}
