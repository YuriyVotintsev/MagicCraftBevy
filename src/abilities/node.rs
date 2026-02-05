use std::collections::HashMap;
use bevy::prelude::*;

use crate::Faction;
use crate::stats::DEFAULT_STATS;
use super::context::TargetInfo;
use super::ids::AbilityId;
use super::activator_support::AbilityEntity;
use super::spawn::SpawnContext;
use super::AbilitySource;

pub fn attach_ability(
    commands: &mut Commands,
    owner: Entity,
    owner_faction: Faction,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let mut entity_commands = commands.spawn((
        AbilitySource::new(ability_id, owner, owner_faction),
        AbilityEntity,
        Name::new(format!("Ability_{:?}", ability_id)),
    ));

    let ctx = SpawnContext {
        ability_id,
        caster: owner,
        caster_position: Vec2::ZERO,
        caster_faction: owner_faction,
        source: TargetInfo::EMPTY,
        target: TargetInfo::EMPTY,
        stats: &DEFAULT_STATS,
        index: 0,
        count: 1,
    };

    for component in &ability_def.components {
        component.insert_component(&mut entity_commands, &ctx);
    }
}

#[derive(Resource, Default)]
pub struct AbilityRegistry {
    abilities: Vec<super::ability_def::AbilityDef>,
    name_to_id: HashMap<String, AbilityId>,
}

impl AbilityRegistry {
    pub fn allocate_id(&mut self, name: &str) -> AbilityId {
        let id = AbilityId(self.abilities.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        id
    }

    pub fn register(&mut self, ability: super::ability_def::AbilityDef) {
        self.abilities.push(ability);
    }

    pub fn get(&self, id: AbilityId) -> Option<&super::ability_def::AbilityDef> {
        self.abilities.get(id.0 as usize)
    }

    pub fn get_id(&self, name: &str) -> Option<AbilityId> {
        self.name_to_id.get(name).copied()
    }
}
