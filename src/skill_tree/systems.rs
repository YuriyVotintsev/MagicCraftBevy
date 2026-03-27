use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::money::PlayerMoney;
use crate::player::Player;
use crate::stats::{DirtyStats, Modifiers};

use super::graph::SkillGraph;

#[derive(Message)]
pub struct AllocateNodeRequest {
    pub node_index: usize,
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
