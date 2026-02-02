use serde::Deserialize;

use super::ids::NodeDefId;
use super::node::NodeDef;
use crate::building_blocks::activators::ActivatorParamsRaw;
use super::ActivatorParams;

#[derive(Debug, Clone, Deserialize)]
pub struct AbilityDefRaw {
    pub id: String,
    pub activator: ActivatorParamsRaw,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
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
            activator_params: ActivatorParams::OnInputParams(crate::building_blocks::activators::on_input::OnInputParams),
            root_action_nodes: vec![],
            nodes: vec![],
        }
    }

    pub fn add_node(&mut self, def: NodeDef) -> NodeDefId {
        let id = NodeDefId(self.nodes.len() as u32);
        self.nodes.push(def);
        id
    }

    pub fn set_activator(&mut self, params: ActivatorParams) {
        self.activator_params = params;
    }

    pub fn set_root_actions(&mut self, roots: Vec<NodeDefId>) {
        self.root_action_nodes = roots;
    }
}
