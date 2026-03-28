use std::collections::HashMap;

use bevy::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::stats::{ModifierDef, ModifierDefRaw, StatId, StatRange, StatRegistry};

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

        for (i, raw) in self.nodes.iter().enumerate() {
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

#[derive(Debug, Clone)]
pub struct PassiveNodeDef {
    pub name: String,
    pub rarity: Rarity,
    pub modifiers: Vec<ModifierDef>,
}

impl PassiveNodeDef {
    pub fn roll_values(&self, rng: &mut impl Rng) -> Vec<(StatId, f32)> {
        self.modifiers
            .iter()
            .flat_map(|m| m.stats.iter())
            .map(|sr| match sr {
                StatRange::Fixed { stat, value } => (*stat, *value),
                StatRange::Range { stat, min, max } => {
                    if (*max - *min).abs() < f32::EPSILON {
                        (*stat, *min)
                    } else {
                        (*stat, rng.random_range(*min..=*max))
                    }
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveNodeDefRaw {
    pub name: String,
    pub rarity: String,
    pub modifiers: Vec<ModifierDefRaw>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RarityWeightsRaw {
    pub center: Vec<u32>,
    pub edge: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassivePoolRaw {
    pub nodes: Vec<PassiveNodeDefRaw>,
    pub rarity_order: Vec<String>,
    pub rarity_weights: RarityWeightsRaw,
}

#[derive(Resource, Debug, Clone)]
pub struct PassiveNodePool {
    pub defs: Vec<PassiveNodeDef>,
    pub by_rarity: HashMap<Rarity, Vec<usize>>,
    pub rarity_order: Vec<Rarity>,
    pub rarity_weights_center: Vec<f32>,
    pub rarity_weights_edge: Vec<f32>,
}

impl PassiveNodePool {
    pub fn from_raw(raw: &PassivePoolRaw, stat_registry: &StatRegistry) -> Self {
        let rarity_map: HashMap<&str, Rarity> = raw
            .rarity_order
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), Rarity(i as u8)))
            .collect();

        let mut defs = Vec::new();
        let mut by_rarity: HashMap<Rarity, Vec<usize>> = HashMap::new();

        for (i, node_raw) in raw.nodes.iter().enumerate() {
            let rarity = *rarity_map
                .get(node_raw.rarity.as_str())
                .unwrap_or_else(|| panic!("Unknown rarity '{}' in node '{}'", node_raw.rarity, node_raw.name));

            let modifiers = node_raw
                .modifiers
                .iter()
                .map(|m| m.resolve(stat_registry))
                .collect();

            defs.push(PassiveNodeDef {
                name: node_raw.name.clone(),
                rarity,
                modifiers,
            });

            by_rarity.entry(rarity).or_default().push(i);
        }

        let rarity_order: Vec<Rarity> = raw
            .rarity_order
            .iter()
            .map(|name| *rarity_map.get(name.as_str()).unwrap())
            .collect();

        let total_center: f32 = raw.rarity_weights.center.iter().sum::<u32>() as f32;
        let total_edge: f32 = raw.rarity_weights.edge.iter().sum::<u32>() as f32;

        let rarity_weights_center: Vec<f32> = raw
            .rarity_weights
            .center
            .iter()
            .map(|&w| w as f32 / total_center)
            .collect();
        let rarity_weights_edge: Vec<f32> = raw
            .rarity_weights
            .edge
            .iter()
            .map(|&w| w as f32 / total_edge)
            .collect();

        Self {
            defs,
            by_rarity,
            rarity_order,
            rarity_weights_center,
            rarity_weights_edge,
        }
    }

    pub fn pick_rarity(&self, t: f32, rng: &mut impl Rng) -> Rarity {
        let weights: Vec<f32> = self
            .rarity_weights_center
            .iter()
            .zip(self.rarity_weights_edge.iter())
            .map(|(&c, &e)| c + (e - c) * t)
            .collect();

        let total: f32 = weights.iter().sum();
        let mut roll = rng.random_range(0.0..total);
        for (i, &w) in weights.iter().enumerate() {
            roll -= w;
            if roll <= 0.0 {
                return self.rarity_order[i];
            }
        }
        *self.rarity_order.last().unwrap()
    }

    pub fn pick_node(&self, rarity: Rarity, rng: &mut impl Rng) -> Option<usize> {
        self.by_rarity
            .get(&rarity)
            .and_then(|indices| indices.choose(rng).copied())
    }
}
