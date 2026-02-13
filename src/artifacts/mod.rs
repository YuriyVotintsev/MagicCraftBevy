use std::collections::HashMap;

use bevy::prelude::*;
use rand::prelude::*;

use crate::stats::StatId;
use crate::wave::WavePhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ArtifactId(pub u32);

impl From<u32> for ArtifactId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<ArtifactId> for u32 {
    fn from(id: ArtifactId) -> Self {
        id.0
    }
}

#[derive(Component)]
pub struct Artifact(pub ArtifactId);

pub struct ArtifactModifier {
    pub stat: StatId,
    pub value: f32,
    pub name: String,
}

pub struct ArtifactDef {
    pub name: String,
    pub price: u32,
    pub modifiers: Vec<ArtifactModifier>,
}

#[derive(serde::Deserialize)]
pub struct ArtifactDefRaw {
    pub id: String,
    pub name: String,
    pub price: u32,
    #[serde(default)]
    pub modifiers: HashMap<String, f32>,
}

#[derive(Resource, Default)]
pub struct ArtifactRegistry {
    artifacts: Vec<ArtifactDef>,
}

impl ArtifactRegistry {
    pub fn register(&mut self, def: ArtifactDef) -> ArtifactId {
        let id = ArtifactId(self.artifacts.len() as u32);
        self.artifacts.push(def);
        id
    }

    pub fn get(&self, id: ArtifactId) -> Option<&ArtifactDef> {
        self.artifacts.get(id.0 as usize)
    }
}

#[derive(Resource)]
pub struct PlayerArtifacts {
    pub slots: Vec<Option<Entity>>,
}

impl Default for PlayerArtifacts {
    fn default() -> Self {
        Self {
            slots: vec![None; 5],
        }
    }
}

impl PlayerArtifacts {
    pub fn buy(&mut self, entity: Entity) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
            *slot = Some(entity);
            true
        } else {
            false
        }
    }

    pub fn sell(&mut self, slot_index: usize) -> Option<Entity> {
        if slot_index < self.slots.len() {
            self.slots[slot_index].take()
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    pub fn equipped(&self) -> Vec<(usize, Entity)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.map(|e| (i, e)))
            .collect()
    }
}

#[derive(Resource, Default)]
pub struct ShopOfferings(pub Vec<Entity>);

#[derive(Resource, Default)]
pub struct AvailableArtifacts(pub Vec<ArtifactId>);

#[derive(Resource)]
pub struct RerollCost(pub u32);

impl Default for RerollCost {
    fn default() -> Self {
        Self(1)
    }
}

pub fn reroll_offerings(
    commands: &mut Commands,
    offerings: &mut ShopOfferings,
    available: &AvailableArtifacts,
) {
    for entity in offerings.0.drain(..) {
        commands.entity(entity).despawn();
    }

    let mut rng = rand::rng();
    let mut ids = available.0.clone();
    ids.shuffle(&mut rng);
    offerings.0 = ids
        .into_iter()
        .take(3)
        .map(|id| commands.spawn(Artifact(id)).id())
        .collect();
}

fn generate_shop_offerings(
    mut commands: Commands,
    mut offerings: ResMut<ShopOfferings>,
    available: Res<AvailableArtifacts>,
) {
    reroll_offerings(&mut commands, &mut offerings, &available);
}

pub struct ArtifactPlugin;

impl Plugin for ArtifactPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArtifactRegistry>()
            .init_resource::<PlayerArtifacts>()
            .init_resource::<ShopOfferings>()
            .init_resource::<AvailableArtifacts>()
            .init_resource::<RerollCost>()
            .add_systems(OnEnter(WavePhase::ShopDelay), generate_shop_offerings);
    }
}
