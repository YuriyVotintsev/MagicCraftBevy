use serde::{Deserialize, Serialize};

use super::ids::{NodeDefId, NodeTypeId};
use super::node::{NodeDef, NodeDefRaw};
use super::ActivatorParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDefRaw {
    pub id: String,
    pub activator: NodeDefRaw,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub activator_type: String,
    pub activator_params: ActivatorParams,
    pub root_action_nodes: Vec<NodeDefId>,
    nodes: Vec<NodeDef>,
}

impl AbilityDef {
    pub fn get_node(&self, id: NodeDefId) -> Option<&NodeDef> {
        self.nodes.get(id.0 as usize)
    }

    pub fn new() -> Self {
        Self {
            activator_type: String::new(),
            activator_params: ActivatorParams::OnInput,
            root_action_nodes: vec![],
            nodes: vec![],
        }
    }

    pub fn add_node(&mut self, def: NodeDef) -> NodeDefId {
        let id = NodeDefId(self.nodes.len() as u32);
        self.nodes.push(def);
        id
    }

    pub fn set_activator(&mut self, type_name: String, params: ActivatorParams) {
        self.activator_type = type_name;
        self.activator_params = params;
    }

    pub fn set_root_actions(&mut self, roots: Vec<NodeDefId>) {
        self.root_action_nodes = roots;
    }

    pub fn has_trigger(&self, node_id: NodeDefId, trigger_type: NodeTypeId) -> bool {
        let Some(node) = self.get_node(node_id) else {
            return false;
        };

        node.children.iter().any(|&child_id| {
            self.get_node(child_id)
                .map(|n| n.node_type == trigger_type)
                .unwrap_or(false)
        })
    }
}
