use bevy::prelude::*;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::bullet::spawn_bullet;
use crate::stats::{
    ComputedStats, DirtyStats, RawStats, StatCalculators, StatId, StatsBundle,
};

const PLAYER_SIZE: f32 = 100.0;
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

fn spawn_player(mut commands: Commands, calculators: Res<StatCalculators>) {
    let mut raw = RawStats::default();
    let mut dirty = DirtyStats::default();

    raw.set(StatId::Strength, 10.0, &mut dirty, &calculators);
    raw.set(StatId::MaxLife, 100.0, &mut dirty, &calculators);
    raw.set(StatId::MaxLifePerStrength, 2.0, &mut dirty, &calculators);
    raw.set(StatId::MovementSpeed, 400.0, &mut dirty, &calculators);
    raw.set(StatId::PhysicalDamage, 10.0, &mut dirty, &calculators);
    raw.set(StatId::ProjectileSpeed, 800.0, &mut dirty, &calculators);
    raw.set(StatId::ProjectileCount, 1.0, &mut dirty, &calculators);
    raw.set(StatId::CritChance, 0.05, &mut dirty, &calculators);
    raw.set(StatId::CritMultiplier, 1.5, &mut dirty, &calculators);

    commands.spawn((
        Player,
        StatsBundle {
            raw,
            computed: ComputedStats::default(),
            dirty,
        },
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
    mut query: Query<(&mut Transform, &ComputedStats), With<Player>>,
) {
    let Ok((mut transform, stats)) = query.single_mut() else {
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
        let speed = stats.get(StatId::MovementSpeed);
        transform.translation.x += direction.x * speed * time.delta_secs();
        transform.translation.y += direction.y * speed * time.delta_secs();
    }
}

fn player_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<&Transform, With<Player>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let direction = (world_pos - player_pos).normalize_or_zero();

    if direction != Vec2::ZERO {
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
