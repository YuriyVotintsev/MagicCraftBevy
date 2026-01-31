use serde::{Deserialize, Serialize};

use super::ids::{AbilityId, NodeDefId, NodeTypeId};
use super::node::{NodeDef, NodeDefRaw};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDefRaw {
    pub id: String,
    #[serde(alias = "trigger")]
    pub root_node: NodeDefRaw,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub root_node: NodeDefId,
    nodes: Vec<NodeDef>,
}

impl AbilityDef {
    pub fn get_node(&self, id: NodeDefId) -> Option<&NodeDef> {
        self.nodes.get(id.0 as usize)
    }

    pub fn new(id: AbilityId) -> Self {
        Self {
            id,
            root_node: NodeDefId(0),
            nodes: vec![],
        }
    }

    pub fn add_node(&mut self, def: NodeDef) -> NodeDefId {
        let id = NodeDefId(self.nodes.len() as u32);
        self.nodes.push(def);
        id
    }

    pub fn set_root_node(&mut self, id: NodeDefId) {
        self.root_node = id;
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
