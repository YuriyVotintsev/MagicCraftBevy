use std::collections::HashMap;
use bevy::prelude::*;

use crate::Faction;
use super::context::TargetInfo;
use super::core_components::{BlueprintId, BlueprintEntity, BlueprintActivationInput, BlueprintActivationCooldown};
use super::SpawnSource;

pub fn spawn_blueprint_entity(
    commands: &mut Commands,
    owner: Entity,
    owner_faction: Faction,
    blueprint_id: BlueprintId,
    initially_pressed: bool,
) {
    commands.spawn((
        SpawnSource {
            blueprint_id,
            caster: TargetInfo::from_entity_and_position(owner, Vec2::ZERO),
            caster_faction: owner_faction,
            source: TargetInfo::EMPTY,
            target: TargetInfo::EMPTY,
            index: 0,
            count: 1,
        },
        BlueprintEntity,
        BlueprintActivationCooldown { timer: 0.0 },
        BlueprintActivationInput { pressed: initially_pressed, target: TargetInfo::EMPTY },
        Name::new(format!("Blueprint_{:?}", blueprint_id)),
    ));
}

#[derive(Resource, Default)]
pub struct BlueprintRegistry {
    blueprints: Vec<super::blueprint_def::BlueprintDef>,
    name_to_id: HashMap<String, BlueprintId>,
    id_to_name: Vec<String>,
}

impl BlueprintRegistry {
    pub fn register(&mut self, name: &str, blueprint: super::blueprint_def::BlueprintDef) -> BlueprintId {
        let id = BlueprintId(self.blueprints.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        self.blueprints.push(blueprint);
        id
    }

    pub fn get(&self, id: BlueprintId) -> Option<&super::blueprint_def::BlueprintDef> {
        self.blueprints.get(id.0 as usize)
    }

    pub fn get_id(&self, name: &str) -> Option<BlueprintId> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_name(&self, id: BlueprintId) -> Option<&str> {
        self.id_to_name.get(id.0 as usize).map(|s| s.as_str())
    }
}
