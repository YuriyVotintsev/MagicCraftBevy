use bevy::prelude::*;
use std::collections::HashSet;

use crate::game_state::GameState;

use super::kind::ArtifactKind;

#[derive(Resource, Default, Debug, Clone)]
pub struct ArtifactInventory {
    pub collected: Vec<ArtifactKind>,
    pub disabled: HashSet<ArtifactKind>,
}

impl ArtifactInventory {
    pub fn active(&self) -> impl Iterator<Item = ArtifactKind> + '_ {
        self.collected
            .iter()
            .copied()
            .filter(|k| !self.disabled.contains(k))
    }

    pub fn add(&mut self, k: ArtifactKind) {
        for r in k.def().replaces {
            self.disabled.insert(*r);
        }
        self.collected.push(k);
    }

    pub fn pop_last(&mut self) -> Option<ArtifactKind> {
        let popped = self.collected.pop()?;
        for r in popped.def().replaces {
            let still_replaced = self
                .collected
                .iter()
                .any(|k| k.def().replaces.contains(r));
            if !still_replaced {
                self.disabled.remove(r);
            }
        }
        Some(popped)
    }

    pub fn contains(&self, k: ArtifactKind) -> bool {
        self.collected.contains(&k)
    }
}

pub fn register(app: &mut App) {
    app.init_resource::<ArtifactInventory>()
        .add_systems(OnEnter(GameState::Playing), reset_inventory);
}

fn reset_inventory(mut inventory: ResMut<ArtifactInventory>) {
    *inventory = ArtifactInventory::default();
}
