use bevy::prelude::*;

use crate::artifact::apply::RebuildPlayerStateEvent;
use crate::artifact::inventory::ArtifactInventory;
use crate::artifact::pool;
use crate::artifact::reroll::RerollState;
use crate::run::BreatherTimer;

pub fn register(app: &mut App) {
    app.add_systems(Update, on_breather_started);
}

fn on_breather_started(
    breather: Option<Res<BreatherTimer>>,
    mut last_active: Local<bool>,
    mut inventory: ResMut<ArtifactInventory>,
    mut reroll: ResMut<RerollState>,
    mut rebuild: MessageWriter<RebuildPlayerStateEvent>,
) {
    let active = breather.is_some();
    if active && !*last_active {
        let mut rng = rand::rng();
        let prev_accepted = inventory.collected.last().copied();
        let drawn = pool::roll_artifact(&inventory, prev_accepted, &mut rng);
        if let Some(k) = drawn {
            inventory.add(k);
            rebuild.write(RebuildPlayerStateEvent);
        }
        *reroll = RerollState {
            available: drawn.is_some(),
            current: drawn,
        };
    }
    if !active && *last_active {
        *reroll = RerollState::default();
    }
    *last_active = active;
}
