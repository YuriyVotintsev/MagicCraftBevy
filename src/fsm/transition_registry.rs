use bevy::prelude::*;

use super::types::TransitionDef;

pub type TransitionAdder = fn(&mut Commands, Entity, &TransitionDef);
pub type TransitionRemover = fn(&mut Commands, Entity);

#[derive(Resource, Default)]
pub struct TransitionRegistry {
    handlers: Vec<(&'static str, TransitionAdder, TransitionRemover)>,
}

impl TransitionRegistry {
    pub fn register(
        &mut self,
        name: &'static str,
        adder: TransitionAdder,
        remover: TransitionRemover,
    ) {
        self.handlers.push((name, adder, remover));
    }

    pub fn add(&self, commands: &mut Commands, entity: Entity, transition: &TransitionDef) {
        let name = transition.type_name();
        if let Some((_, adder, _)) = self.handlers.iter().find(|(n, _, _)| *n == name) {
            adder(commands, entity, transition);
        }
    }

    pub fn remove(&self, commands: &mut Commands, entity: Entity, transition: &TransitionDef) {
        let name = transition.type_name();
        if let Some((_, _, remover)) = self.handlers.iter().find(|(n, _, _)| *n == name) {
            remover(commands, entity);
        }
    }
}
