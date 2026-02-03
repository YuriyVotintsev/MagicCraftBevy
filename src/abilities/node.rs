use std::collections::HashMap;
use bevy::prelude::*;

use super::ids::AbilityId;
use super::{AbilityInstance, spawn_activator};

pub fn attach_ability(
    commands: &mut Commands,
    owner: Entity,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let mut entity_commands = commands.spawn((
        AbilityInstance { ability_id, owner },
        Name::new(format!("Ability_{:?}", ability_id)),
    ));

    spawn_activator(
        &mut entity_commands,
        &ability_def.activator_params,
    );
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
