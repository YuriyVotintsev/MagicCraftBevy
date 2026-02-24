use bevy::prelude::*;

use crate::stats::StatId;

use super::types::Rarity;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub def_index: usize,
    pub position: Vec2,
    pub rarity: Rarity,
    pub rolled_values: Vec<(StatId, f32)>,
    pub allocated: bool,
}

impl GraphNode {
    pub fn is_start(&self) -> bool {
        self.def_index == usize::MAX
    }
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub a: usize,
    pub b: usize,
}

#[derive(Resource, Debug, Clone)]
pub struct SkillGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub adjacency: Vec<Vec<usize>>,
    pub start_node: usize,
    pub seed: u64,
}

impl SkillGraph {
    pub fn is_allocatable(&self, node_idx: usize) -> bool {
        let node = &self.nodes[node_idx];
        if node.allocated {
            return false;
        }
        self.adjacency[node_idx]
            .iter()
            .any(|&neighbor| self.nodes[neighbor].allocated)
    }

    pub fn allocate(&mut self, node_idx: usize) {
        self.nodes[node_idx].allocated = true;
    }

    pub fn allocated_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.allocated).count()
    }

    pub fn allocatable_nodes(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.nodes.len()).filter(|&i| self.is_allocatable(i))
    }

    pub fn neighbors(&self, node_idx: usize) -> &[usize] {
        &self.adjacency[node_idx]
    }
}
