use std::collections::HashMap;
use bevy::prelude::*;

use crate::Faction;
use crate::stats::DEFAULT_STATS;
use super::context::TargetInfo;
use super::ids::AbilityId;
use super::core_components::{AbilityEntity, AbilityInput, AbilityCooldown};
use super::AbilitySource;

pub fn attach_ability(
    commands: &mut Commands,
    owner: Entity,
    owner_faction: Faction,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
    initially_pressed: bool,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let cooldown = ability_def.cooldown.eval(
        &AbilitySource {
            ability_id,
            caster: TargetInfo::EMPTY,
            caster_faction: owner_faction,
            source: TargetInfo::EMPTY,
            target: TargetInfo::EMPTY,
            index: 0,
            count: 1,
        },
        &DEFAULT_STATS,
    );

    commands.spawn((
        AbilitySource {
            ability_id,
            caster: TargetInfo::from_entity_and_position(owner, Vec2::ZERO),
            caster_faction: owner_faction,
            source: TargetInfo::EMPTY,
            target: TargetInfo::EMPTY,
            index: 0,
            count: 1,
        },
        AbilityEntity,
        AbilityCooldown { cooldown, timer: 0.0 },
        AbilityInput { pressed: initially_pressed, target: TargetInfo::EMPTY },
        Name::new(format!("Ability_{:?}", ability_id)),
    ));
}

#[derive(Resource, Default)]
pub struct AbilityRegistry {
    abilities: Vec<super::ability_def::AbilityDef>,
    name_to_id: HashMap<String, AbilityId>,
}

impl AbilityRegistry {
    pub fn register(&mut self, name: &str, ability: super::ability_def::AbilityDef) -> AbilityId {
        let id = AbilityId(self.abilities.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.abilities.push(ability);
        id
    }

    pub fn get(&self, id: AbilityId) -> Option<&super::ability_def::AbilityDef> {
        self.abilities.get(id.0 as usize)
    }

    pub fn get_id(&self, name: &str) -> Option<AbilityId> {
        self.name_to_id.get(name).copied()
    }
}
