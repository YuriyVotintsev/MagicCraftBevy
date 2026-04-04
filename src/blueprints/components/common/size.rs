use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};

#[derive(Resource)]
pub struct SizeScaleLayer(pub ScaleLayerId);

#[blueprint_component]
pub struct Size {
    pub value: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_layer);
    app.add_systems(Last, (init_scale_modifiers, sync_size_to_scale).chain());
}

fn register_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(SizeScaleLayer(registry.register()));
}

fn init_scale_modifiers(
    mut commands: Commands,
    query: Query<Entity, (With<Size>, Without<ScaleModifiers>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(ScaleModifiers::default());
    }
}

fn sync_size_to_scale(
    layer: Res<SizeScaleLayer>,
    mut query: Query<(&Size, &mut ScaleModifiers), Or<(Added<Size>, Added<ScaleModifiers>)>>,
) {
    for (size, mut modifiers) in &mut query {
        modifiers.set(layer.0, Vec3::splat(size.value / 2.0));
    }
}
