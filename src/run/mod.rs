use bevy::prelude::*;

mod combat_scope;
mod death;
mod lifecycle;
mod pill;
mod run_scope;

pub use combat_scope::{CombatScoped, SkipDeathShrink};
pub use death::PlayerDying;
pub use lifecycle::{wave_duration, BreatherTimer, RunState, StartWaveEvent};
pub use run_scope::RunScoped;

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        combat_scope::register(app);
        run_scope::register(app);
        lifecycle::register(app);
        death::register(app);
        pill::register(app);
    }
}
