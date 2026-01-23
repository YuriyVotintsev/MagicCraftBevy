use bevy::prelude::*;
use rand::Rng;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::player::Player;

pub const ENEMY_SIZE: f32 = 90.0;
const ENEMY_SPEED: f32 = 150.0;
const ENEMY_SPAWN_INTERVAL: f32 = 1.5;

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(
            ENEMY_SPAWN_INTERVAL,
            TimerMode::Repeating,
        )))
        .add_systems(Update, (spawn_enemies, move_enemies, clamp_enemies));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::rng();
        let half_width = ARENA_WIDTH / 2.0 - ENEMY_SIZE / 2.0;
        let half_height = ARENA_HEIGHT / 2.0 - ENEMY_SIZE / 2.0;

        let x = rng.random_range(-half_width..half_width);
        let y = rng.random_range(-half_height..half_height);

        commands.spawn((
            Enemy,
            Mesh2d(meshes.add(Triangle2d::new(
                Vec2::new(0.0, ENEMY_SIZE / 2.0),
                Vec2::new(-ENEMY_SIZE / 2.0, -ENEMY_SIZE / 2.0),
                Vec2::new(ENEMY_SIZE / 2.0, -ENEMY_SIZE / 2.0),
            ))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.2, 0.2)))),
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}

fn move_enemies(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for mut enemy_transform in &mut enemy_query {
        let direction = (player_transform.translation - enemy_transform.translation)
            .truncate()
            .normalize_or_zero();

        enemy_transform.translation.x += direction.x * ENEMY_SPEED * time.delta_secs();
        enemy_transform.translation.y += direction.y * ENEMY_SPEED * time.delta_secs();
    }
}

fn clamp_enemies(mut query: Query<&mut Transform, With<Enemy>>) {
    let half_width = ARENA_WIDTH / 2.0 - ENEMY_SIZE / 2.0;
    let half_height = ARENA_HEIGHT / 2.0 - ENEMY_SIZE / 2.0;

    for mut transform in &mut query {
        transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
        transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
    }
}
