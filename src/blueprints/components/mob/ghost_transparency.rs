use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::shadow::Shadow;
use crate::blueprints::components::common::sprite::{CircleSprite, SquareSprite, TriangleSprite, CapsuleSprite};
use crate::health_material::{HealthMaterial, HealthMaterialLink};
use crate::player::Player;
use crate::schedule::GameSet;
use crate::summoning::SummoningCircle;
use crate::GameState;

#[blueprint_component]
pub struct GhostTransparency {
    #[raw(default = 200.0)]
    pub visible_distance: ScalarExpr,
    #[raw(default = 800.0)]
    pub invisible_distance: ScalarExpr,
}

#[derive(Component)]
pub struct GhostAlpha {
    pub value: f32,
}

#[derive(Component)]
pub struct StoredCollisionLayers(pub CollisionLayers);

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            init,
            update_ghost_alpha,
            toggle_ghost_collider,
        )
            .chain()
            .in_set(GameSet::MobAI)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        PostUpdate,
        (apply_ghost_alpha_to_children, apply_ghost_alpha_to_circle)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init(mut commands: Commands, query: Query<Entity, Added<GhostTransparency>>) {
    for entity in &query {
        commands.entity(entity).insert(GhostAlpha { value: 0.0 });
    }
}

fn update_ghost_alpha(
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &GhostTransparency, &mut GhostAlpha), Without<Player>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    let player_pos = crate::coord::to_2d(player_tf.translation);

    for (transform, ghost, mut alpha) in &mut query {
        let pos = crate::coord::to_2d(transform.translation);
        let dist = pos.distance(player_pos);
        let t = ((dist - ghost.visible_distance) / (ghost.invisible_distance - ghost.visible_distance))
            .clamp(0.0, 1.0);
        alpha.value = 1.0 - t;
    }
}

fn apply_ghost_alpha_to_children(
    ghost_query: Query<
        (&GhostAlpha, &Children, Option<&HealthMaterialLink>),
        Without<SummoningCircle>,
    >,
    shadow_query: Query<(&Shadow, &MeshMaterial3d<StandardMaterial>)>,
    sprite_query: Query<
        &MeshMaterial3d<StandardMaterial>,
        Or<(With<CircleSprite>, With<SquareSprite>, With<TriangleSprite>, With<CapsuleSprite>)>,
    >,
    sprite_health_query: Query<
        &MeshMaterial3d<HealthMaterial>,
        Or<(With<CircleSprite>, With<SquareSprite>, With<TriangleSprite>, With<CapsuleSprite>)>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut health_materials: ResMut<Assets<HealthMaterial>>,
) {
    for (alpha, children, health_link) in &ghost_query {
        for child in children.iter() {
            if let Ok((shadow, mat_handle)) = shadow_query.get(child) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let mut color = material.base_color.to_srgba();
                    color.alpha = shadow.opacity * alpha.value;
                    material.base_color = color.into();
                }
                continue;
            }

            if let Some(link) = health_link {
                if sprite_health_query.get(child).is_ok() {
                    if let Some(material) = health_materials.get_mut(&link.handle) {
                        material.data.alpha = alpha.value;
                    }
                    continue;
                }
            }

            if let Ok(mat_handle) = sprite_query.get(child) {
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let mut color = material.base_color.to_srgba();
                    color.alpha = alpha.value;
                    material.base_color = color.into();
                    material.alpha_mode = if alpha.value < 1.0 {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    };
                }
            }
        }
    }
}

fn apply_ghost_alpha_to_circle(
    query: Query<
        (&GhostAlpha, &MeshMaterial3d<StandardMaterial>, &SummoningCircle),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut emitter_query: Query<&mut crate::particles::ParticleEmitter>,
) {
    for (alpha, mat_handle, circle) in &query {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            let mut color = material.base_color.to_srgba();
            color.alpha = 0.7 * alpha.value;
            material.base_color = color.into();
        }

        if let Some(emitter_entity) = circle.emitter {
            if let Ok(mut emitter) = emitter_query.get_mut(emitter_entity) {
                let handle = emitter.material_override.get_or_insert_with(|| {
                    materials.add(StandardMaterial {
                        base_color: crate::palette::color("enemy"),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })
                });
                if let Some(material) = materials.get_mut(handle) {
                    let mut color = material.base_color.to_srgba();
                    color.alpha = alpha.value;
                    material.base_color = color.into();
                }
            }
        }
    }
}

fn toggle_ghost_collider(
    mut commands: Commands,
    mut query: Query<
        (Entity, &GhostAlpha, &mut CollisionLayers, Option<&StoredCollisionLayers>),
        Changed<GhostAlpha>,
    >,
) {
    for (entity, alpha, mut layers, stored) in &mut query {
        if alpha.value == 0.0 && stored.is_none() {
            commands.entity(entity).insert(StoredCollisionLayers(*layers));
            *layers = CollisionLayers::NONE;
        } else if alpha.value > 0.0 {
            if let Some(stored) = stored {
                *layers = stored.0;
                commands.entity(entity).remove::<StoredCollisionLayers>();
            }
        }
    }
}
