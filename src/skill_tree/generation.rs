use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

use super::graph::{GraphEdge, GraphNode, SkillGraph};
use super::types::{PassiveNodePool, Rarity};

const TARGET_NODES: usize = 400;
const GRAPH_RADIUS: f32 = 2000.0;
const MIN_DISTANCE: f32 = 140.0;
const PRUNE_LENGTH_FACTOR: f32 = 1.5;

pub fn generate_skill_graph(pool: &PassiveNodePool, seed: u64) -> SkillGraph {
    let mut rng = StdRng::seed_from_u64(seed);

    let points = poisson_disk_sampling(GRAPH_RADIUS, MIN_DISTANCE, &mut rng);
    info!("Skill tree: generated {} points", points.len());

    let triangulation_edges = delaunay_edges(&points);
    info!("Skill tree: {} Delaunay edges", triangulation_edges.len());

    let pruned_edges = prune_edges(&points, &triangulation_edges);
    info!("Skill tree: {} edges after pruning", pruned_edges.len());

    let start_node = 0;
    let mut nodes = Vec::with_capacity(points.len());

    nodes.push(GraphNode {
        def_index: usize::MAX,
        position: points[0],
        rarity: Rarity(0),
        rolled_values: vec![],
        allocated: true,
    });

    let max_dist = points
        .iter()
        .map(|p| p.length())
        .fold(0.0f32, f32::max)
        .max(1.0);

    for &pos in points.iter().skip(1) {
        let t = (pos.length() / max_dist).clamp(0.0, 1.0);
        let rarity = pool.pick_rarity(t, &mut rng);
        let def_index = pool.pick_node(rarity, &mut rng).unwrap_or(0);
        let rolled_values = pool.defs[def_index].roll_values(&mut rng);

        nodes.push(GraphNode {
            def_index,
            position: pos,
            rarity,
            rolled_values,
            allocated: false,
        });
    }

    let mut adjacency = vec![vec![]; nodes.len()];
    for edge in &pruned_edges {
        adjacency[edge.a].push(edge.b);
        adjacency[edge.b].push(edge.a);
    }

    SkillGraph {
        nodes,
        edges: pruned_edges,
        adjacency,
        start_node,
        seed,
    }
}

fn poisson_disk_sampling(radius: f32, min_dist: f32, rng: &mut impl Rng) -> Vec<Vec2> {
    let cell_size = min_dist / std::f32::consts::SQRT_2;
    let grid_extent = (radius * 2.0 / cell_size).ceil() as i32 + 1;
    let grid_offset = grid_extent / 2;

    let mut grid: Vec<Vec<Option<usize>>> =
        vec![vec![None; grid_extent as usize]; grid_extent as usize];
    let mut points = Vec::with_capacity(TARGET_NODES + 50);
    let mut active = Vec::new();

    let center = Vec2::ZERO;
    points.push(center);
    active.push(0);
    let gx = grid_offset;
    let gy = grid_offset;
    grid[gx as usize][gy as usize] = Some(0);

    let max_attempts = 30;

    while !active.is_empty() {
        let active_idx = rng.random_range(0..active.len());
        let point_idx = active[active_idx];
        let point = points[point_idx];

        let mut found = false;
        for _ in 0..max_attempts {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let dist = rng.random_range(min_dist..min_dist * 2.0);
            let candidate = point + Vec2::new(angle.cos(), angle.sin()) * dist;

            if candidate.length() > radius {
                continue;
            }

            let cx = ((candidate.x / cell_size) + grid_offset as f32) as i32;
            let cy = ((candidate.y / cell_size) + grid_offset as f32) as i32;

            if cx < 0 || cy < 0 || cx >= grid_extent || cy >= grid_extent {
                continue;
            }

            let mut too_close = false;
            'outer: for dx in -2..=2 {
                for dy in -2..=2 {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx >= 0 && ny >= 0 && nx < grid_extent && ny < grid_extent {
                        if let Some(neighbor_idx) = grid[nx as usize][ny as usize] {
                            if points[neighbor_idx].distance(candidate) < min_dist {
                                too_close = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }

            if !too_close {
                let new_idx = points.len();
                points.push(candidate);
                active.push(new_idx);
                grid[cx as usize][cy as usize] = Some(new_idx);
                found = true;
                break;
            }
        }

        if !found {
            active.swap_remove(active_idx);
        }
    }

    points
}

fn delaunay_edges(points: &[Vec2]) -> Vec<GraphEdge> {
    if points.len() < 3 {
        if points.len() == 2 {
            return vec![GraphEdge { a: 0, b: 1 }];
        }
        return vec![];
    }

    let del_points: Vec<delaunator::Point> = points
        .iter()
        .map(|p| delaunator::Point {
            x: p.x as f64,
            y: p.y as f64,
        })
        .collect();

    let Some(result) = delaunator::triangulate(&del_points) else {
        return vec![];
    };

    let mut edge_set = std::collections::HashSet::new();
    let mut edges = Vec::new();

    for tri in result.triangles.chunks(3) {
        let pairs = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in pairs {
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_set.insert(key) {
                edges.push(GraphEdge { a, b });
            }
        }
    }

    edges
}

fn prune_edges(points: &[Vec2], edges: &[GraphEdge]) -> Vec<GraphEdge> {
    let mut lengths: Vec<f32> = edges
        .iter()
        .map(|e| points[e.a].distance(points[e.b]))
        .collect();
    lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median = lengths[lengths.len() / 2];
    let threshold = median * PRUNE_LENGTH_FACTOR;

    let mut sorted_edges: Vec<(usize, f32)> = edges
        .iter()
        .enumerate()
        .map(|(i, e)| (i, points[e.a].distance(points[e.b])))
        .collect();
    sorted_edges.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut keep = vec![true; edges.len()];
    let n = points.len();

    for &(edge_idx, length) in &sorted_edges {
        if length <= threshold {
            break;
        }

        keep[edge_idx] = false;

        if !is_connected_without(edges, &keep, n) {
            keep[edge_idx] = true;
        }
    }

    edges
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, e)| e.clone())
        .collect()
}

fn is_connected_without(edges: &[GraphEdge], keep: &[bool], n: usize) -> bool {
    if n == 0 {
        return true;
    }

    let mut adj = vec![vec![]; n];
    for (i, edge) in edges.iter().enumerate() {
        if keep[i] {
            adj[edge.a].push(edge.b);
            adj[edge.b].push(edge.a);
        }
    }

    let mut visited = vec![false; n];
    let mut stack = vec![0usize];
    visited[0] = true;
    let mut count = 1;

    while let Some(node) = stack.pop() {
        for &neighbor in &adj[node] {
            if !visited[neighbor] {
                visited[neighbor] = true;
                count += 1;
                stack.push(neighbor);
            }
        }
    }

    count == n
}
