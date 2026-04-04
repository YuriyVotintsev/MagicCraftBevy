use bevy::prelude::*;
use smallvec::SmallVec;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScaleLayerId(u8);

#[derive(Resource, Default)]
pub struct ScaleLayerRegistry {
    count: u8,
}

impl ScaleLayerRegistry {
    pub fn register(&mut self) -> ScaleLayerId {
        let id = ScaleLayerId(self.count);
        self.count += 1;
        id
    }
}

#[derive(Component, Default)]
pub struct ScaleModifiers {
    factors: SmallVec<[(ScaleLayerId, Vec3); 4]>,
}

impl ScaleModifiers {
    pub fn set(&mut self, id: ScaleLayerId, value: Vec3) {
        if let Some(entry) = self.factors.iter_mut().find(|(k, _)| *k == id) {
            entry.1 = value;
        } else {
            self.factors.push((id, value));
        }
    }

    pub fn product(&self) -> Vec3 {
        self.factors
            .iter()
            .fold(Vec3::ONE, |acc, (_, f)| acc * *f)
    }
}

fn resolve_scale(mut query: Query<(&ScaleModifiers, &mut Transform), Changed<ScaleModifiers>>) {
    for (modifiers, mut transform) in &mut query {
        transform.scale = modifiers.product();
    }
}

pub struct CompositeScalePlugin;

impl Plugin for CompositeScalePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScaleLayerRegistry>()
            .add_systems(PostUpdate, resolve_scale);
    }
}
