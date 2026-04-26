use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::GameLayer;
use crate::game_state::GameState;
use crate::run::{CombatScoped, SkipDeathShrink};

use super::size::CurrentArenaSize;

pub(super) const WALL_HEIGHT: f32 = 200.0;
pub(super) const WALL_THICKNESS: f32 = 20.0;
const WALL_SEGMENTS: u32 = 64;

#[derive(Component)]
pub struct Wall;

pub fn register(app: &mut App) {
    app.add_systems(Update, sync_walls.run_if(in_state(GameState::Playing)));
}

fn sync_walls(
    mut commands: Commands,
    arena_size: Option<Res<CurrentArenaSize>>,
    walls: Query<Entity, With<Wall>>,
) {
    let Some(arena_size) = arena_size else { return };
    if arena_size.radius <= 0.0 {
        return;
    }
    let walls_empty = walls.iter().next().is_none();
    if walls_empty {
        spawn_walls(&mut commands, arena_size.radius);
    } else if arena_size.is_changed() {
        for e in &walls {
            commands.entity(e).despawn();
        }
        spawn_walls(&mut commands, arena_size.radius);
    }
}

pub(super) fn spawn_walls(commands: &mut Commands, radius: f32) {
    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);
    let n = WALL_SEGMENTS;
    let segment_angle = std::f32::consts::TAU / n as f32;
    let chord = 2.0 * radius * (segment_angle * 0.5).sin();
    let segment_len = chord + WALL_THICKNESS;

    for i in 0..n {
        let angle = segment_angle * i as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        commands.spawn((
            Name::new("WallSegment"),
            Wall,
            Transform::from_translation(Vec3::new(x, WALL_HEIGHT / 2.0, z))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 - angle)),
            Collider::cuboid(segment_len, WALL_HEIGHT, WALL_THICKNESS),
            CollisionMargin(5.0),
            RigidBody::Static,
            wall_layers,
            CombatScoped,
            SkipDeathShrink,
        ));
    }
}
