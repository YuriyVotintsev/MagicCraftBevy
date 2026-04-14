use bevy::prelude::*;

mod phase;
mod spawn;
mod state;
mod summoning;

pub use phase::{CombatPhase, WavePhase};
pub use spawn::EnemySpawnPool;
pub use state::{InvulnerableStack, WaveEnemy};
pub use summoning::{RiseFromGround, SummoningCircle};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        state::register(app);
        spawn::register(app);
        summoning::register(app);
    }
}
