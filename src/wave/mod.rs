use bevy::prelude::*;

pub mod config;
mod phase;
mod spawn;
mod state;
mod summoning;

pub use config::WavesConfig;
pub use phase::{CombatPhase, WavePhase};
pub use spawn::EnemySpawnPool;
pub use state::{InvulnerableStack, WaveEnemy};
pub use summoning::RiseFromGround;

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        state::register(app);
        spawn::register(app);
        summoning::register(app);
    }
}
