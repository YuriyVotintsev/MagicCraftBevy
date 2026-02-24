use bevy::prelude::*;

use crate::player::Player;
use crate::stats::{DirtyStats, Modifiers};

use super::generation::generate_skill_graph;
use super::graph::SkillGraph;
use super::types::PassiveNodePool;

#[derive(Resource, Default)]
pub struct SkillPoints(pub u32);

#[derive(Message)]
pub struct AllocateNodeRequest {
    pub node_index: usize,
}

pub fn generate_skill_tree(mut commands: Commands, pool: Option<Res<PassiveNodePool>>) {
    let Some(pool) = pool else {
        warn!("PassiveNodePool not available, skipping skill tree generation");
        return;
    };

    let seed = rand::random::<u64>();
    let graph = generate_skill_graph(&pool, seed);
    info!(
        "Generated skill tree: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );
    commands.insert_resource(graph);
    commands.insert_resource(SkillPoints(0));
}

pub fn grant_skill_points(mut skill_points: ResMut<SkillPoints>) {
    skill_points.0 += 3;
    info!("Granted 3 skill points, total: {}", skill_points.0);
}

pub fn handle_allocate_node(
    mut events: MessageReader<AllocateNodeRequest>,
    graph: Option<ResMut<SkillGraph>>,
    skill_points: Option<ResMut<SkillPoints>>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats), With<Player>>,
) {
    let (Some(mut graph), Some(mut skill_points)) = (graph, skill_points) else {
        return;
    };

    for event in events.read() {
        let idx = event.node_index;

        if skill_points.0 < 1 {
            continue;
        }
        if !graph.is_allocatable(idx) {
            continue;
        }

        graph.allocate(idx);
        skill_points.0 -= 1;

        for (mut modifiers, mut dirty) in &mut player_query {
            for &(stat, value) in &graph.nodes[idx].rolled_values {
                modifiers.add(stat, value, None);
                dirty.mark(stat);
            }
        }

        info!(
            "Allocated skill tree node {}, points remaining: {}",
            idx, skill_points.0
        );
    }
}

pub fn cleanup_skill_tree(mut commands: Commands) {
    commands.remove_resource::<SkillGraph>();
    commands.remove_resource::<SkillPoints>();
}
