use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use super::super::ability::lifetime::Lifetime;

#[blueprint_component]
pub struct FadeOut {}

#[derive(Component)]
pub struct FadeOutState {
    pub total: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, (init_fade_out, update_fade_out).chain());
}

fn init_fade_out(
    mut commands: Commands,
    query: Query<(Entity, &Lifetime), (With<FadeOut>, Without<FadeOutState>)>,
) {
    for (entity, lifetime) in &query {
        commands.entity(entity).insert(FadeOutState {
            total: lifetime.remaining,
        });
    }
}

fn update_fade_out(
    query: Query<(&Lifetime, &FadeOutState, &MeshMaterial3d<StandardMaterial>), With<FadeOut>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (lifetime, state, mat_handle) in &query {
        let alpha = (lifetime.remaining / state.total).clamp(0.0, 1.0);
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let mut color = mat.base_color.to_srgba();
            color.alpha = alpha;
            mat.base_color = color.into();
            mat.alpha_mode = AlphaMode::Blend;
        }
    }
}
