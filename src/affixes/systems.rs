use bevy::prelude::*;

use crate::player::{Player, SpellSlot};
use crate::GameState;

use super::components::{Affixes, SlotOwner, SpellSlotTag};

pub fn spawn_spell_slots(mut commands: Commands, player_query: Query<Entity, Added<Player>>) {
    for player_entity in &player_query {
        for slot in [SpellSlot::Active, SpellSlot::Passive, SpellSlot::Defensive] {
            commands.spawn((
                SpellSlotTag(slot),
                Affixes::default(),
                SlotOwner(player_entity),
                DespawnOnExit(GameState::Playing),
            ));
        }
    }
}
