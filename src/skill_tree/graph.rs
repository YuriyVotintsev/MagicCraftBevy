use bevy::prelude::*;

use crate::stats::StatId;

use super::types::Rarity;

#[derive(Resource)]
pub struct GridSettings {
    pub size: f32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self { size: 100.0 }
    }
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub position: Vec2,
    pub grid_cell: IVec2,
    pub rarity: Rarity,
    pub rolled_values: Vec<(StatId, f32)>,
    pub level: u32,
    pub max_level: u32,
    pub name: String,
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
}

impl SkillGraph {
    pub fn is_allocatable(&self, node_idx: usize) -> bool {
        let node = &self.nodes[node_idx];
        if node.level >= node.max_level {
            return false;
        }
        if node_idx == self.start_node {
            return true;
        }
        self.adjacency[node_idx]
            .iter()
            .any(|&neighbor| self.nodes[neighbor].level > 0)
    }

    pub fn allocate(&mut self, node_idx: usize) {
        self.nodes[node_idx].level += 1;
    }

    pub fn allocated_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.level > 0).count()
    }

    pub fn allocatable_nodes(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.nodes.len()).filter(|&i| self.is_allocatable(i))
    }

    pub fn neighbors(&self, node_idx: usize) -> &[usize] {
        &self.adjacency[node_idx]
    }
}
