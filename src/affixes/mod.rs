mod components;
mod operations;
mod orbs;
mod registry;
mod systems;
mod types;

pub use components::{Affixes, SlotOwner, SpellSlotTag};
pub use operations::{apply_alteration, apply_augmentation, apply_chaos, sync_affix_modifiers};
pub use orbs::{OrbBehavior, OrbDef, OrbDefRaw, OrbFlowState, OrbId, OrbRegistry};
pub use registry::AffixRegistry;
pub use types::{Affix, AffixDef, AffixDefRaw};

use bevy::prelude::*;

use crate::GameState;

pub struct AffixPlugin;

impl Plugin for AffixPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AffixRegistry>()
            .init_resource::<OrbRegistry>()
            .init_resource::<OrbFlowState>()
            .add_systems(OnEnter(GameState::Playing), reset_affixes)
            .add_systems(
                Update,
                systems::spawn_spell_slots.run_if(in_state(GameState::Playing)),
            );
    }
}

fn reset_affixes(mut orb_flow: ResMut<OrbFlowState>) {
    *orb_flow = Default::default();
}
