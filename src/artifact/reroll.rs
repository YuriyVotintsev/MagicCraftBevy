use bevy::prelude::*;

use super::apply::RebuildPlayerStateEvent;
use super::card::ArtifactCardData;
use super::inventory::ArtifactInventory;
use super::kind::ArtifactKind;
use super::pool;
use crate::run::BreatherTimer;
use crate::ui::widgets::ReleasedButtons;

#[derive(Component)]
pub struct RerollButton;

#[derive(Resource, Default)]
pub struct RerollState {
    pub available: bool,
    pub current: Option<ArtifactKind>,
}

pub fn register(app: &mut App) {
    app.init_resource::<RerollState>().add_systems(
        Update,
        reroll_button_system.run_if(resource_exists::<BreatherTimer>),
    );
}

fn reroll_button_system(
    buttons: ReleasedButtons<RerollButton>,
    mut state: ResMut<RerollState>,
    mut inventory: ResMut<ArtifactInventory>,
    mut rebuild: MessageWriter<RebuildPlayerStateEvent>,
    mut card_q: Query<&mut ArtifactCardData>,
) {
    buttons.for_each(|_| {
        if !state.available {
            return;
        }
        let Some(prev) = state.current else { return };
        inventory.pop_last();

        let prev_accepted = inventory.collected.last().copied();
        let mut rng = rand::rng();
        let new = pool::roll_artifact_excluding(&inventory, prev_accepted, prev, &mut rng);
        if let Some(k) = new {
            inventory.add(k);
            rebuild.write(RebuildPlayerStateEvent);
            state.current = Some(k);
        } else {
            state.current = None;
        }
        state.available = false;
        if let Ok(mut data) = card_q.single_mut() {
            data.kind = state.current;
            data.refresh = true;
        }
    });
}
