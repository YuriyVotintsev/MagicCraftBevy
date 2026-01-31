use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::stats::ComputedStats;
use super::ids::{NodeTypeId, NodeDefId, ParamId, AbilityId};
use super::param::{ParamValue, ParamValueRaw};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Trigger,
    Action,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodeKind::Trigger => write!(f, "Trigger"),
            NodeKind::Action => write!(f, "Action"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeDef {
    pub kind: NodeKind,
    pub node_type: NodeTypeId,
    pub params: HashMap<ParamId, ParamValue>,
    pub children: Vec<NodeDefId>,
}

impl NodeDef {
    pub fn get_param<'a>(&'a self, name: &str, registry: &NodeRegistry) -> Option<&'a ParamValue> {
        let id = registry.get_param_id(name)?;
        self.params.get(&id)
    }

    pub fn get_f32(&self, name: &str, stats: &ComputedStats, registry: &NodeRegistry) -> Option<f32> {
        Some(self.get_param(name, registry)?.evaluate_f32(stats))
    }

    pub fn get_i32(&self, name: &str, stats: &ComputedStats, registry: &NodeRegistry) -> Option<i32> {
        Some(self.get_param(name, registry)?.evaluate_i32(stats))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeDefRaw {
    Full(String, HashMap<String, ParamValueRaw>, Vec<NodeDefRaw>),
    NoChildren(String, HashMap<String, ParamValueRaw>),
    NoParams(String, Vec<NodeDefRaw>),
    OnlyName(String),
}

impl NodeDefRaw {
    pub fn destructure(self) -> (String, HashMap<String, ParamValueRaw>, Vec<NodeDefRaw>) {
        match self {
            NodeDefRaw::Full(n, p, c) => (n, p, c),
            NodeDefRaw::NoChildren(n, p) => (n, p, vec![]),
            NodeDefRaw::NoParams(n, c) => (n, HashMap::new(), c),
            NodeDefRaw::OnlyName(n) => (n, HashMap::new(), vec![]),
        }
    }
}

pub trait NodeHandler: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn kind(&self) -> NodeKind;

    fn add_to_entity(
        &self,
        _commands: &mut Commands,
        _entity: Entity,
        _ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &NodeRegistry,
    ) {
        panic!("{} is not a Trigger node and cannot be added to entity", self.name());
    }

    fn register_input_systems(&self, _app: &mut App) {
        panic!("{} is not a Trigger node", self.name());
    }

    fn register_execution_system(&self, _app: &mut App) {
        panic!("{} is not an Action node", self.name());
    }

    fn register_behavior_systems(&self, _app: &mut App) {}
}

#[derive(Resource, Default)]
pub struct NodeRegistry {
    name_to_id: HashMap<String, NodeTypeId>,
    id_to_name: Vec<String>,
    handlers: Vec<Box<dyn NodeHandler>>,
    kinds: Vec<NodeKind>,
    param_name_to_id: HashMap<String, ParamId>,
    param_id_to_name: Vec<String>,
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

    pub fn get(&self, id: NodeTypeId) -> Option<&dyn NodeHandler> {
        self.handlers.get(id.0 as usize).map(|h| h.as_ref())
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
}

pub fn attach_ability(
    commands: &mut Commands,
    entity: Entity,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
    node_registry: &NodeRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let Some(root_node) = ability_def.get_node(ability_def.root_node) else {
        return;
    };

    let Some(handler) = node_registry.get(root_node.node_type) else {
        warn!(
            "Unknown node type: {:?}",
            root_node.node_type
        );
        return;
    };

    handler.add_to_entity(
        commands,
        entity,
        ability_id,
        &root_node.params,
        node_registry,
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
