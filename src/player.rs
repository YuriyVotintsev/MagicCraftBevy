use bevy::prelude::*;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::bullet::spawn_bullet;

const PLAYER_SIZE: f32 = 100.0;
const PLAYER_SPEED: f32 = 400.0;
const BULLET_SPEED: f32 = 800.0;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, player_shooting, clamp_player));
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        transform.translation.x += direction.x * PLAYER_SPEED * time.delta_secs();
        transform.translation.y += direction.y * PLAYER_SPEED * time.delta_secs();
    }
}

fn player_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = query.single() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        spawn_bullet(
            &mut commands,
            &mut meshes,
            &mut materials,
            player_transform.translation,
            direction * BULLET_SPEED,
        );
    }
}

fn clamp_player(mut query: Query<&mut Transform, With<Player>>) {
    let half_width = ARENA_WIDTH / 2.0 - PLAYER_SIZE / 2.0;
    let half_height = ARENA_HEIGHT / 2.0 - PLAYER_SIZE / 2.0;

    if let Ok(mut transform) = query.single_mut() {
        transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
        transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
    }
}
