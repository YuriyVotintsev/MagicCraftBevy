use bevy::prelude::*;
use std::collections::HashMap;

use super::ids::{ActivatorTypeId, EffectTypeId, AbilityId, ParamId};
use super::context::AbilityContext;
use super::activator_def::{ActivatorState, ActivationResult, ActivatorDef};
use super::effect_def::{EffectDef, ParamValue};
use super::ability_def::AbilityDef;
use super::components::AbilityInput;

pub trait Activator: Send + Sync + 'static {
    fn check(
        &self,
        def: &ActivatorDef,
        state: &mut ActivatorState,
        ctx: &mut AbilityContext,
        input: &AbilityInput,
    ) -> ActivationResult;
}

pub trait EffectExecutor: Send + Sync + 'static {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    );
}

#[derive(Resource, Default)]
pub struct ActivatorRegistry {
    name_to_id: HashMap<String, ActivatorTypeId>,
    id_to_name: Vec<String>,
    activators: Vec<Box<dyn Activator>>,
}

impl ActivatorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A: Activator>(&mut self, name: &str, activator: A) -> ActivatorTypeId {
        let id = ActivatorTypeId(self.activators.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        self.activators.push(Box::new(activator));
        id
    }

    pub fn get_id(&self, name: &str) -> Option<ActivatorTypeId> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_name(&self, id: ActivatorTypeId) -> Option<&str> {
        self.id_to_name.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn get(&self, id: ActivatorTypeId) -> Option<&dyn Activator> {
        self.activators.get(id.0 as usize).map(|a| a.as_ref())
    }

    pub fn check(
        &self,
        def: &ActivatorDef,
        state: &mut ActivatorState,
        ctx: &mut AbilityContext,
        input: &AbilityInput,
    ) -> ActivationResult {
        if let Some(activator) = self.get(def.activator_type) {
            activator.check(def, state, ctx, input)
        } else {
            ActivationResult::NotReady
        }
    }
}

#[derive(Resource, Default)]
pub struct EffectRegistry {
    name_to_id: HashMap<String, EffectTypeId>,
    id_to_name: Vec<String>,
    executors: Vec<Box<dyn EffectExecutor>>,
    param_name_to_id: HashMap<String, ParamId>,
    param_id_to_name: Vec<String>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<E: EffectExecutor>(&mut self, name: &str, executor: E) -> EffectTypeId {
        let id = EffectTypeId(self.executors.len() as u32);
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        self.executors.push(Box::new(executor));
        id
    }

    pub fn get_id(&self, name: &str) -> Option<EffectTypeId> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_name(&self, id: EffectTypeId) -> Option<&str> {
        self.id_to_name.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn get(&self, id: EffectTypeId) -> Option<&dyn EffectExecutor> {
        self.executors.get(id.0 as usize).map(|e| e.as_ref())
    }

    pub fn get_or_insert_param_id(&mut self, name: &str) -> ParamId {
        if let Some(&id) = self.param_name_to_id.get(name) {
            return id;
        }
        let id = ParamId(self.param_id_to_name.len() as u32);
        self.param_name_to_id.insert(name.to_string(), id);
        self.param_id_to_name.push(name.to_string());
        id
    }

    pub fn get_param_id(&self, name: &str) -> Option<ParamId> {
        self.param_name_to_id.get(name).copied()
    }

    pub fn get_param_name(&self, id: ParamId) -> Option<&str> {
        self.param_id_to_name.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn execute(&self, def: &EffectDef, ctx: &AbilityContext, commands: &mut Commands) {
        if let Some(executor) = self.get(def.effect_type) {
            executor.execute(def, ctx, commands, self);
        }
    }
}

#[derive(Resource, Default)]
pub struct AbilityRegistry {
    abilities: HashMap<AbilityId, AbilityDef>,
    name_to_id: HashMap<String, AbilityId>,
    next_id: u32,
}

impl AbilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, def: AbilityDef) -> AbilityId {
        let id = def.id;
        self.abilities.insert(id, def);
        id
    }

    pub fn get(&self, id: AbilityId) -> Option<&AbilityDef> {
        self.abilities.get(&id)
    }

    pub fn get_id(&self, name: &str) -> Option<AbilityId> {
        self.name_to_id.get(name).copied()
    }

    pub fn allocate_id(&mut self, name: &str) -> AbilityId {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }
        let id = AbilityId(self.next_id);
        self.next_id += 1;
        self.name_to_id.insert(name.to_string(), id);
        id
    }
}
