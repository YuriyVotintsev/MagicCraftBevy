use bevy::prelude::*;

use super::types::BehaviourDef;

pub type BehaviourAdder = fn(&mut Commands, Entity, &BehaviourDef);
pub type BehaviourRemover = fn(&mut Commands, Entity);

#[derive(Resource, Default)]
pub struct BehaviourRegistry {
    handlers: Vec<(&'static str, BehaviourAdder, BehaviourRemover)>,
}

impl BehaviourRegistry {
    pub fn register(
        &mut self,
        name: &'static str,
        adder: BehaviourAdder,
        remover: BehaviourRemover,
    ) {
        self.handlers.push((name, adder, remover));
    }

    pub fn add(&self, commands: &mut Commands, entity: Entity, behaviour: &BehaviourDef) {
        let name = behaviour.type_name();
        if let Some((_, adder, _)) = self.handlers.iter().find(|(n, _, _)| *n == name) {
            adder(commands, entity, behaviour);
        }
    }

    pub fn remove(&self, commands: &mut Commands, entity: Entity, behaviour: &BehaviourDef) {
        let name = behaviour.type_name();
        if let Some((_, _, remover)) = self.handlers.iter().find(|(n, _, _)| *n == name) {
            remover(commands, entity);
        }
    }
}
