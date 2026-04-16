use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::GameState;
use super::super::components::{
    BobbingAnimation, CapsuleSprite, Caster, CircleSprite, Collider, DynamicBody, FindNearestEnemy,
    Health, MeleeAttacker, OnDeathParticles, SelfMoving, Shadow, Shape as ColliderShape, Size,
    Sprite, SpriteShape, Target,
};
use super::super::player::Player;
use crate::faction::Faction;
use crate::health_material::{HealthMaterial, HealthMaterialLink};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat, StatCalculators};
use crate::wave::SummoningCircle;

use super::spawn::{compute_stats, current_max_life, enemy_sprite_color};

#[derive(Clone, Deserialize, Debug)]
pub struct GhostStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub mass: f32,
    pub melee_range: f32,
    pub melee_cooldown: f32,
    pub visible_distance: f32,
    pub invisible_distance: f32,
}

#[derive(Component)]
pub struct GhostTransparency {
    pub visible_distance: f32,
    pub invisible_distance: f32,
}

#[derive(Component)]
pub struct GhostAlpha {
    pub value: f32,
}

#[derive(Component)]
pub struct StoredCollisionLayers(pub CollisionLayers);

#[derive(Component)]
pub struct MoveToward {}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            init_ghost_transparency,
            update_ghost_alpha,
            toggle_ghost_collider,
            move_toward_system,
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
    app.add_observer(|on: On<Remove, MoveToward>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

pub fn spawn_ghost(
    commands: &mut Commands,
    pos: Vec2,
    s: &GhostStats,
    calculators: &StatCalculators,
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        calculators,
        &[(Stat::MovementSpeedFlat, s.speed), (Stat::MaxLifeFlat, s.hp), (Stat::PhysicalDamageFlat, s.damage)],
    );
    let hp = current_max_life(&computed);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: s.mass },
        Health { current: hp },
        GhostTransparency { visible_distance: s.visible_distance, invisible_distance: s.invisible_distance },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        MoveToward {},
        MeleeAttacker::new(s.melee_cooldown, s.melee_range),
    )).id();

    commands.entity(id).insert((
        Caster(id),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn((
            Sprite {
                color: enemy_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, elevation: 0.5, half_length: 0.5,
            },
            BobbingAnimation { amplitude: 0.2, speed: 2.0, base_elevation: 0.5 },
        ));
    });

    id
}

fn init_ghost_transparency(mut commands: Commands, query: Query<Entity, Added<GhostTransparency>>) {
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
        Or<(With<CircleSprite>, With<CapsuleSprite>)>,
    >,
    sprite_health_query: Query<
        &MeshMaterial3d<HealthMaterial>,
        Or<(With<CircleSprite>, With<CapsuleSprite>)>,
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

fn move_toward_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut LinearVelocity, &ComputedStats, Option<&Target>), (With<MoveToward>, Without<crate::wave::RiseFromGround>)>,
    transforms: Query<&Transform, Without<MoveToward>>,
) {
    for (entity, transform, mut velocity, stats, target) in &mut query {
        let Some(target) = target else {
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<SelfMoving>();
            continue;
        };
        let target_entity = target.0;
        let Ok(target_transform) = transforms.get(target_entity) else {
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<SelfMoving>();
            continue;
        };
        let speed = stats.get(Stat::MovementSpeed);
        let direction = crate::coord::to_2d(target_transform.translation - transform.translation);

        velocity.0 = if direction.length_squared() > 1.0 {
            commands.entity(entity).insert(SelfMoving);
            crate::coord::ground_vel(direction.normalize() * speed)
        } else {
            commands.entity(entity).remove::<SelfMoving>();
            Vec3::ZERO
        };
    }
}
