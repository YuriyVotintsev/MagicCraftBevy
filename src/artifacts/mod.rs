mod registry;
mod resources;
mod systems;
mod types;

pub use registry::ArtifactRegistry;
pub use resources::{Artifact, AvailableArtifacts, PlayerArtifacts, RerollCost, ShopOfferings};
pub use systems::reroll_offerings;
pub use types::{ArtifactDef, ArtifactDefRaw, ArtifactId};

use bevy::prelude::*;

use crate::wave::WavePhase;
use systems::generate_shop_offerings;

pub struct ArtifactPlugin;

impl Plugin for ArtifactPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArtifactRegistry>()
            .init_resource::<PlayerArtifacts>()
            .init_resource::<ShopOfferings>()
            .init_resource::<AvailableArtifacts>()
            .init_resource::<RerollCost>()
            .add_systems(OnEnter(WavePhase::ShopDelay), generate_shop_offerings);
    }
}
