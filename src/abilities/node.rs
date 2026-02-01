use std::collections::HashMap;
use bevy::prelude::*;

use super::ids::{NodeTypeId, NodeDefId, AbilityId};
use super::params::NodeParams;
use super::{AbilityInstance, spawn_activator};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Trigger,
    Action,
    Activator,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeKind::Trigger => write!(f, "Trigger"),
            NodeKind::Action => write!(f, "Action"),
            NodeKind::Activator => write!(f, "Activator"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeDef {
    pub node_type: NodeTypeId,
    pub params: NodeParams,
    pub children: Vec<NodeDefId>,
}

pub trait NodeHandler: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn kind(&self) -> NodeKind;
}

#[derive(Resource, Default)]
pub struct NodeRegistry {
    name_to_id: HashMap<String, NodeTypeId>,
    id_to_name: Vec<String>,
    handlers: Vec<Box<dyn NodeHandler>>,
    kinds: Vec<NodeKind>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: Box<dyn NodeHandler>) -> NodeTypeId {
        let name = handler.name().to_string();
        if self.name_to_id.contains_key(&name) {
            panic!("Handler with name '{}' already registered", name);
        }
        let kind = handler.kind();
        let id = NodeTypeId(self.handlers.len() as u32);

        self.name_to_id.insert(name.clone(), id);
        self.id_to_name.push(name);
        self.kinds.push(kind);
        self.handlers.push(handler);

        id
    }

    pub fn get_id(&self, name: &str) -> Option<NodeTypeId> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_kind(&self, id: NodeTypeId) -> NodeKind {
        self.kinds[id.0 as usize]
    }
}

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
