use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::GameLayer;

use super::size::CurrentArenaSize;

pub(super) const WALL_HEIGHT: f32 = 200.0;
pub(super) const WALL_THICKNESS: f32 = 20.0;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub(super) enum WallSide { North, South, West, East }

pub fn register(app: &mut App) {
    app.add_systems(Update, update_walls);
}

pub(super) fn spawn_walls(commands: &mut Commands, start_width: f32, start_height: f32) {
    let start_hw = start_width / 2.0;
    let start_hh = start_height / 2.0;
    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    let walls = [
        ("NorthWall", WallSide::North, Vec3::new(0.0, WALL_HEIGHT / 2.0, -start_hh), Vec3::new(start_hw * 2.0 + WALL_THICKNESS, WALL_HEIGHT, WALL_THICKNESS)),
        ("SouthWall", WallSide::South, Vec3::new(0.0, WALL_HEIGHT / 2.0, start_hh), Vec3::new(start_hw * 2.0 + WALL_THICKNESS, WALL_HEIGHT, WALL_THICKNESS)),
        ("WestWall", WallSide::West, Vec3::new(-start_hw, WALL_HEIGHT / 2.0, 0.0), Vec3::new(WALL_THICKNESS, WALL_HEIGHT, start_hh * 2.0 + WALL_THICKNESS)),
        ("EastWall", WallSide::East, Vec3::new(start_hw, WALL_HEIGHT / 2.0, 0.0), Vec3::new(WALL_THICKNESS, WALL_HEIGHT, start_hh * 2.0 + WALL_THICKNESS)),
    ];

    for (name, side, pos, size) in walls {
        commands.spawn((
            Name::new(name),
            Wall,
            side,
            Transform::from_translation(pos),
            Collider::cuboid(size.x, size.y, size.z),
            CollisionMargin(5.0),
            RigidBody::Static,
            wall_layers,
        ));
    }
}

fn update_walls(
    arena_size: Option<Res<CurrentArenaSize>>,
    mut query: Query<(&WallSide, &mut Transform, &mut Collider)>,
) {
    let Some(arena_size) = arena_size else { return };
    if !arena_size.is_changed() {
        return;
    }
    let half_w = arena_size.half_w();
    let half_h = arena_size.half_h();

    for (side, mut transform, mut collider) in &mut query {
        match side {
            WallSide::North => {
                transform.translation.z = -half_h;
                *collider = Collider::cuboid(half_w * 2.0 + WALL_THICKNESS, WALL_HEIGHT, WALL_THICKNESS);
            }
            WallSide::South => {
                transform.translation.z = half_h;
                *collider = Collider::cuboid(half_w * 2.0 + WALL_THICKNESS, WALL_HEIGHT, WALL_THICKNESS);
            }
            WallSide::West => {
                transform.translation.x = -half_w;
                *collider = Collider::cuboid(WALL_THICKNESS, WALL_HEIGHT, half_h * 2.0 + WALL_THICKNESS);
            }
            WallSide::East => {
                transform.translation.x = half_w;
                *collider = Collider::cuboid(WALL_THICKNESS, WALL_HEIGHT, half_h * 2.0 + WALL_THICKNESS);
            }
        }
    }
}
