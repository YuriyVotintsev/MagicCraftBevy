use avian3d::prelude::*;
use bevy::prelude::*;

use super::{CapsuleShape, CircleShape, Shadow};
use crate::health_material::{HealthMaterial, HealthMaterialLink};

#[derive(Component)]
pub struct Fade {
    pub alpha: f32,
}

impl Default for Fade {
    fn default() -> Self {
        Self { alpha: 1.0 }
    }
}

#[derive(Component)]
pub struct FadeCollisionToggle;

#[derive(Component)]
pub struct FadeBase(pub f32);

impl Default for FadeBase {
    fn default() -> Self { Self(1.0) }
}

#[derive(Component)]
struct StoredCollisionLayers(CollisionLayers);

pub fn register_systems(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (apply_fade_to_self, apply_fade_to_children, toggle_fade_collision),
    );
}

fn apply_fade_to_self(
    query: Query<(&Fade, &MeshMaterial3d<StandardMaterial>, Option<&FadeBase>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (fade, mat_handle, base) in &query {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            let base_alpha = base.map(|b| b.0).unwrap_or(1.0);
            let out_alpha = base_alpha * fade.alpha;
            let mut color = material.base_color.to_srgba();
            color.alpha = out_alpha;
            material.base_color = color.into();
            material.alpha_mode = if out_alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
        }
    }
}

fn apply_fade_to_children(
    parents: Query<(&Fade, &Children, Option<&HealthMaterialLink>)>,
    shadow_query: Query<(&Shadow, &MeshMaterial3d<StandardMaterial>)>,
    shape_query: Query<
        &MeshMaterial3d<StandardMaterial>,
        Or<(With<CircleShape>, With<CapsuleShape>)>,
    >,
    shape_health_query: Query<
        &MeshMaterial3d<HealthMaterial>,
        Or<(With<CircleShape>, With<CapsuleShape>)>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut health_materials: ResMut<Assets<HealthMaterial>>,
) {
    for (fade, children, health_link) in &parents {
        for child in children.iter() {
            if let Ok((shadow, mat_handle)) = shadow_query.get(child) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let mut color = material.base_color.to_srgba();
                    color.alpha = shadow.opacity * fade.alpha;
                    material.base_color = color.into();
                }
                continue;
            }

            if let Some(link) = health_link {
                if shape_health_query.get(child).is_ok() {
                    if let Some(material) = health_materials.get_mut(&link.handle) {
                        material.data.alpha = fade.alpha;
                    }
                    continue;
                }
            }

            if let Ok(mat_handle) = shape_query.get(child) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let mut color = material.base_color.to_srgba();
                    color.alpha = fade.alpha;
                    material.base_color = color.into();
                    material.alpha_mode = if fade.alpha < 1.0 {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    };
                }
            }
        }
    }
}

fn toggle_fade_collision(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Fade, &mut CollisionLayers, Option<&StoredCollisionLayers>),
        (With<FadeCollisionToggle>, Changed<Fade>),
    >,
) {
    for (entity, fade, mut layers, stored) in &mut query {
        if fade.alpha == 0.0 && stored.is_none() {
            commands.entity(entity).insert(StoredCollisionLayers(*layers));
            *layers = CollisionLayers::NONE;
        } else if fade.alpha > 0.0 && stored.is_some() {
            if let Some(stored) = stored {
                *layers = stored.0;
                commands.entity(entity).remove::<StoredCollisionLayers>();
            }
        }
    }
}
