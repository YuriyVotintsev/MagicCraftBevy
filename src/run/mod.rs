use bevy::prelude::*;

mod coin;
mod death;
mod lifecycle;
mod money;

pub use death::PlayerDying;
pub use lifecycle::RunState;
pub use money::PlayerMoney;

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        lifecycle::register(app);
        death::register(app);
        coin::register(app);
        money::register(app);
    }
}
