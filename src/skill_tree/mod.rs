pub mod generation;
pub mod graph;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::schedule::ShopSet;
use crate::wave::WavePhase;

pub struct SkillTreePlugin;

impl Plugin for SkillTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<systems::AllocateNodeRequest>()
            .add_systems(OnEnter(GameState::Playing), systems::generate_skill_tree)
            .add_systems(OnEnter(WavePhase::Shop), systems::grant_skill_points)
            .add_systems(
                Update,
                systems::handle_allocate_node
                    .in_set(ShopSet::Process)
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(OnExit(GameState::Playing), systems::cleanup_skill_tree);
    }
}
