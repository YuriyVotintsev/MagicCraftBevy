use bevy::prelude::*;

mod coin;
mod combat_scope;
mod death;
mod lifecycle;
mod money;

pub use combat_scope::{CombatScoped, SkipDeathShrink};
pub use death::PlayerDying;
pub use lifecycle::RunState;
pub use money::PlayerMoney;

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        combat_scope::register(app);
        lifecycle::register(app);
        death::register(app);
        coin::register(app);
        money::register(app);
    }
}
