use bevy::prelude::*;
use rand::Rng;

use crate::actors::components::{Health, Shadow, Shape, ShapeColor, ShapeKind, Size};
use crate::actors::{death_system, DeathEvent, Player};
use crate::game_state::GameState;
use crate::palette;
use crate::schedule::{GameSet, PostGameSet};
use crate::stats::{ComputedStats, Stat};
use crate::wave::{CombatPhase, WaveEnemy};

use super::combat_scope::CombatScoped;

const PILL_SIZE: f32 = 60.0;
const PILL_HEAL_PCT: f32 = 0.10;
const PILL_COOLDOWN_MIN: f32 = 10.0;
const PILL_COOLDOWN_MAX: f32 = 15.0;
const PILL_ATTRACTION_DURATION: f32 = 0.5;
const PILL_BASE_PICKUP_RADIUS: f32 = 250.0;

#[derive(Component)]
pub struct HealthPill;

#[derive(Component)]
pub struct PillAttracted {
    origin: Vec2,
    elapsed: f32,
}

#[derive(Resource)]
pub struct PillDropTimer(pub f32);

fn random_cooldown() -> f32 {
    rand::rng().random_range(PILL_COOLDOWN_MIN..PILL_COOLDOWN_MAX)
}

pub fn register(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), reset_pill_timer)
        .add_systems(
            Update,
            tick_pill_timer
                .run_if(in_state(CombatPhase::Running))
                .in_set(GameSet::WaveManagement),
        )
        .add_systems(
            PostUpdate,
            spawn_pill_on_death
                .in_set(PostGameSet)
                .after(death_system)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (attract_pills, move_pills, collect_pills)
                .chain()
                .in_set(GameSet::WaveManagement),
        );
}

fn reset_pill_timer(mut commands: Commands) {
    commands.insert_resource(PillDropTimer(random_cooldown()));
}

fn tick_pill_timer(time: Res<Time>, timer: Option<ResMut<PillDropTimer>>) {
    let Some(mut timer) = timer else { return };
    if timer.0 > 0.0 {
        timer.0 = (timer.0 - time.delta_secs()).max(0.0);
    }
}

fn spawn_pill_on_death(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    wave_enemy_query: Query<&Transform, With<WaveEnemy>>,
    timer: Option<ResMut<PillDropTimer>>,
) {
    let Some(mut timer) = timer else { return };
    for event in death_events.read() {
        if timer.0 > 0.0 {
            continue;
        }
        let Ok(transform) = wave_enemy_query.get(event.entity) else {
            continue;
        };
        let position = crate::coord::to_2d(transform.translation);
        spawn_pill(&mut commands, position);
        timer.0 = random_cooldown();
    }
}

fn spawn_pill(commands: &mut Commands, position: Vec2) {
    let (r, g, b) = palette::lookup("coin").unwrap_or((0.6, 0.7, 0.3));
    commands
        .spawn((
            Name::new("HealthPill"),
            HealthPill,
            Transform::from_translation(crate::coord::ground_pos(position)),
            Visibility::default(),
            Size { value: PILL_SIZE },
            CombatScoped,
        ))
        .with_children(|p| {
            p.spawn(Shadow);
            p.spawn(Shape {
                color: ShapeColor { r, g, b, a: 1.0, flash: None },
                kind: ShapeKind::Circle,
                position: Vec2::ZERO,
                elevation: 0.5,
                half_length: 0.5,
            });
        });
}

fn attract_pills(
    mut commands: Commands,
    player_query: Query<(&Transform, &ComputedStats), With<Player>>,
    pills: Query<(Entity, &Transform), (With<HealthPill>, Without<PillAttracted>)>,
) {
    let Ok((player_transform, stats)) = player_query.single() else { return };
    let player_pos = crate::coord::to_2d(player_transform.translation);
    let radius = stats.apply(Stat::PickupRadius, PILL_BASE_PICKUP_RADIUS).max(0.0);
    let radius_sq = radius * radius;
    for (entity, t) in &pills {
        let pos = crate::coord::to_2d(t.translation);
        if player_pos.distance_squared(pos) > radius_sq {
            continue;
        }
        commands.entity(entity).insert(PillAttracted {
            origin: pos,
            elapsed: 0.0,
        });
    }
}

fn move_pills(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut pills: Query<(&mut Transform, &mut PillAttracted), (With<HealthPill>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = crate::coord::to_2d(player_transform.translation);
    let dt = time.delta_secs();
    for (mut t, mut a) in &mut pills {
        a.elapsed += dt;
        let f = (a.elapsed / PILL_ATTRACTION_DURATION).min(1.0);
        let eased = f * f;
        let pos = a.origin.lerp(player_pos, eased);
        let g = crate::coord::ground_pos(pos);
        t.translation.x = g.x;
        t.translation.z = g.z;
    }
}

fn collect_pills(
    mut commands: Commands,
    pills: Query<(Entity, &PillAttracted), With<HealthPill>>,
    mut player_query: Query<(&mut Health, &ComputedStats), With<Player>>,
) {
    let Ok((mut health, computed)) = player_query.single_mut() else { return };
    for (entity, a) in &pills {
        if a.elapsed >= PILL_ATTRACTION_DURATION {
            let max = computed.final_of(Stat::MaxLife).max(1.0);
            health.current = (health.current + max * PILL_HEAL_PCT).min(max);
            commands.entity(entity).despawn();
        }
    }
}
