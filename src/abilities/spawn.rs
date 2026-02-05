use bevy::prelude::*;

use crate::Faction;
use crate::stats::ComputedStats;
use super::context::TargetInfo;
use super::eval_context::EvalContext;
use super::entity_def::EntityDef;
use super::{AbilitySource, ids::AbilityId};

pub struct SpawnContext<'a> {
    pub ability_id: AbilityId,
    pub caster: Entity,
    pub caster_position: Vec2,
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
            caster: TargetInfo::from_entity_and_position(self.caster, self.caster_position),
            source: self.source,
            target: self.target,
            stats: self.stats,
            index: self.index,
            count: self.count,
        }
    }
}

pub fn spawn_entity(commands: &mut Commands, entity_def: &EntityDef, ctx: &SpawnContext) -> Entity {
    let mut ec = commands.spawn((
        AbilitySource::new(ctx.ability_id, ctx.caster, ctx.caster_faction),
        ctx.caster_faction,
    ));

    for component in &entity_def.components {
        component.insert_component(&mut ec, ctx);
    }

    ec.id()
}

pub fn spawn_entity_def(commands: &mut Commands, entity_def: &EntityDef, ctx: &SpawnContext) -> Vec<Entity> {
    let count = entity_def.count
        .as_ref()
        .map(|c| c.eval(&ctx.eval_context()).max(1.0) as usize)
        .unwrap_or(1);

    let mut entities = Vec::with_capacity(count);
    for i in 0..count {
        let spawn_ctx = SpawnContext {
            ability_id: ctx.ability_id,
            caster: ctx.caster,
            caster_position: ctx.caster_position,
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
