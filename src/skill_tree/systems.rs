use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::money::PlayerMoney;
use crate::player::Player;
use crate::stats::{DirtyStats, Modifiers};

use super::generation::generate_skill_graph;
use super::graph::SkillGraph;
use super::types::PassiveNodePool;

#[derive(Message)]
pub struct AllocateNodeRequest {
    pub node_index: usize,
}

pub fn generate_skill_tree(
    mut commands: Commands,
    pool: Option<Res<PassiveNodePool>>,
    existing_graph: Option<Res<SkillGraph>>,
) {
    if existing_graph.is_some() {
        return;
    }

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
}

pub fn handle_allocate_node(
    mut events: MessageReader<AllocateNodeRequest>,
    graph: Option<ResMut<SkillGraph>>,
    mut money: ResMut<PlayerMoney>,
    balance: Res<GameBalance>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats), With<Player>>,
) {
    let Some(mut graph) = graph else {
        return;
    };

    let cost = balance.run.node_cost;

    for event in events.read() {
        let idx = event.node_index;

        if !money.can_afford(cost) {
            continue;
        }
        if !graph.is_allocatable(idx) {
            continue;
        }

        graph.allocate(idx);
        money.spend(cost);

        for (mut modifiers, mut dirty) in &mut player_query {
            for &(stat, value) in &graph.nodes[idx].rolled_values {
                modifiers.add(stat, value, None);
                dirty.mark(stat);
            }
        }
    }
}
