use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::stats::{ModifierDefRaw, StatId, StatRange, StatRegistry};

use super::graph::{GraphEdge, GraphNode, SkillGraph};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillTreeNodeRaw {
    pub name: String,
    #[serde(default)]
    pub position: (i32, i32),
    #[serde(default = "default_max_level")]
    pub max_level: u32,
    #[serde(default)]
    pub modifiers: Vec<ModifierDefRaw>,
}

fn default_max_level() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillTreeDefRaw {
    #[serde(default = "default_grid_size")]
    pub grid_size: f32,
    pub nodes: Vec<SkillTreeNodeRaw>,
    pub edges: Vec<(usize, usize)>,
}

fn default_grid_size() -> f32 {
    100.0
}

impl SkillTreeDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> (SkillGraph, f32) {
        let grid_size = self.grid_size;
        let mut nodes = Vec::with_capacity(self.nodes.len());

        for raw in &self.nodes {
            let grid_cell = IVec2::new(raw.position.0, raw.position.1);

            let rolled_values: Vec<(StatId, f32)> = raw
                .modifiers
                .iter()
                .flat_map(|m| {
                    let def = m.resolve(stat_registry);
                    def.stats.into_iter().map(|sr| match sr {
                        StatRange::Fixed { stat, value } => (stat, value),
                        StatRange::Range { stat, min, max } => (stat, (min + max) / 2.0),
                    })
                })
                .collect();

            nodes.push(GraphNode {
                position: Vec2::new(grid_cell.x as f32 * grid_size, grid_cell.y as f32 * grid_size),
                grid_cell,
                rarity: Rarity(0),
                rolled_values,
                level: 0,
                max_level: raw.max_level,
                name: raw.name.clone(),
            });
        }

        let edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .map(|&(a, b)| GraphEdge { a, b })
            .collect();

        let mut adjacency = vec![Vec::new(); nodes.len()];
        for edge in &edges {
            adjacency[edge.a].push(edge.b);
            adjacency[edge.b].push(edge.a);
        }

        (SkillGraph {
            start_node: 0,
            nodes,
            edges,
            adjacency,
        }, grid_size)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Rarity(pub u8);
