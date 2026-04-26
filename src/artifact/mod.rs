use bevy::prelude::*;

mod apply;
mod card;
mod drop;
mod effect;
mod exotic;
mod inventory;
mod kind;
mod pool;
mod reroll;
mod status;
mod wave_end;

pub use apply::{apply_inventory_to_player, RebuildPlayerStateEvent};
pub use effect::{ExoticKind, OnHitEffectStack};
pub use inventory::ArtifactInventory;
pub use kind::ArtifactKind;
pub use status::{Burning, Frozen};

pub struct ArtifactPlugin;

impl Plugin for ArtifactPlugin {
    fn build(&self, app: &mut App) {
        inventory::register(app);
        apply::register(app);
        drop::register(app);
        card::register(app);
        reroll::register(app);
        wave_end::register(app);
        status::register(app);
        exotic::register(app);
    }
}
