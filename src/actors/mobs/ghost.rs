use avian3d::prelude::*;
use bevy::prelude::*;

use crate::GameState;
use crate::balance::MobCommonStats;
use super::super::components::{
    BobbingAnimation, Fade, FadeCollisionToggle, MeleeAttacker, SelfMoving, Shape, ShapeKind,
};
use super::super::player::Player;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, ModifierKind, Stat, StatCalculators};

use super::spawn::{enemy_shape_color, spawn_enemy_core, EnemyBody, WaveModifiers};

const GHOST_MELEE_RANGE: f32 = 80.0;
const GHOST_MELEE_COOLDOWN: f32 = 1.0;
pub const GHOST_VISIBLE_DISTANCE: f32 = 150.0;
pub const GHOST_INVISIBLE_DISTANCE: f32 = 400.0;

#[derive(Component)]
pub struct GhostTransparency {
    pub visible_distance: f32,
    pub invisible_distance: f32,
}

#[derive(Component)]
pub struct MoveToward {}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            init_ghost_transparency,
            update_ghost_fade,
            move_toward_system,
        )
            .chain()
            .in_set(GameSet::MobAI)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_observer(|on: On<Remove, MoveToward>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

pub fn spawn_ghost(
    commands: &mut Commands,
    pos: Vec2,
    s: &MobCommonStats,
    calculators: &StatCalculators,
    wave_mods: WaveModifiers,
) -> Entity {
    let speed = s.speed.unwrap_or(0.0);
    let mass = s.mass.unwrap_or(1.0);
    let id = spawn_enemy_core(
        commands,
        pos,
        calculators,
        &[
            (Stat::MovementSpeed, ModifierKind::Flat, speed),
            (Stat::MaxLife, ModifierKind::Flat, s.hp),
            (Stat::PhysicalDamage, ModifierKind::Flat, s.damage),
        ],
        s.size,
        EnemyBody::Dynamic { mass },
        "enemy_death",
        wave_mods,
    );

    commands.entity(id).insert((
        GhostTransparency {
            visible_distance: GHOST_VISIBLE_DISTANCE,
            invisible_distance: GHOST_INVISIBLE_DISTANCE,
        },
        MoveToward {},
        MeleeAttacker::new(GHOST_MELEE_COOLDOWN, GHOST_MELEE_RANGE),
    ));

    commands.entity(id).with_children(|p| {
        p.spawn((
            Shape {
                color: enemy_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 0.5, half_length: 0.5,
            },
            BobbingAnimation { amplitude: 0.2, speed: 2.0, base_elevation: 0.5 },
        ));
    });

    id
}

fn init_ghost_transparency(mut commands: Commands, query: Query<Entity, Added<GhostTransparency>>) {
    for entity in &query {
        commands.entity(entity).insert((Fade { alpha: 0.0 }, FadeCollisionToggle));
    }
}

fn update_ghost_fade(
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &GhostTransparency, &mut Fade), Without<Player>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    let player_pos = crate::coord::to_2d(player_tf.translation);

    for (transform, ghost, mut fade) in &mut query {
        let pos = crate::coord::to_2d(transform.translation);
        let dist = pos.distance(player_pos);
        let t = ((dist - ghost.visible_distance) / (ghost.invisible_distance - ghost.visible_distance))
            .clamp(0.0, 1.0);
        fade.alpha = 1.0 - t;
    }
}

fn move_toward_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut LinearVelocity, &ComputedStats), (With<MoveToward>, Without<crate::wave::RiseFromGround>)>,
    player: Option<Single<&Transform, (With<Player>, Without<MoveToward>)>>,
) {
    let Some(player) = player else {
        for (entity, _, mut velocity, _) in &mut query {
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<SelfMoving>();
        }
        return;
    };
    for (entity, transform, mut velocity, stats) in &mut query {
        let speed = stats.final_of(Stat::MovementSpeed);
        let direction = crate::coord::to_2d(player.translation - transform.translation);

        velocity.0 = if direction.length_squared() > 1.0 {
            commands.entity(entity).insert(SelfMoving);
            crate::coord::ground_vel(direction.normalize() * speed)
        } else {
            commands.entity(entity).remove::<SelfMoving>();
            Vec3::ZERO
        };
    }
}
