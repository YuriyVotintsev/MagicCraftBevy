use bevy::prelude::*;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::fsm::MobType;

const MOB_SIZE: f32 = 30.0;

pub const BULLET_SIZE: f32 = 15.0;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_bullets, bullet_enemy_collision));
    }
}

pub fn spawn_bullet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec3,
    velocity: Vec2,
) {
    commands.spawn((
        Bullet,
        Velocity(velocity),
        Mesh2d(meshes.add(Circle::new(BULLET_SIZE))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(1.0, 1.0, 0.0)))),
        Transform::from_translation(position),
    ));
}

fn move_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Velocity), With<Bullet>>,
) {
    let half_width = ARENA_WIDTH / 2.0;
    let half_height = ARENA_HEIGHT / 2.0;

    for (entity, mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();

        if transform.translation.x.abs() > half_width
            || transform.translation.y.abs() > half_height
        {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    mob_query: Query<(Entity, &Transform), With<MobType>>,
) {
    for (bullet_entity, bullet_transform) in &bullet_query {
        for (mob_entity, mob_transform) in &mob_query {
            let distance = bullet_transform
                .translation
                .truncate()
                .distance(mob_transform.translation.truncate());

            if distance < (BULLET_SIZE + MOB_SIZE) / 2.0 {
                commands.entity(bullet_entity).despawn();
                commands.entity(mob_entity).despawn();
            }
        }
    }
}
