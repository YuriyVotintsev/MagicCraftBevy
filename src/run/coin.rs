use bevy::prelude::*;

use crate::actors::{death_system, CircleShape, DeathEvent, Shadow};
use crate::actors::Player;
use crate::balance::Globals;
use crate::palette;
use super::combat_scope::CombatScoped;
use super::money::PlayerMoney;
use crate::schedule::{GameSet, PostGameSet};
use crate::stats::{ComputedStats, Stat};
use crate::wave::WaveEnemy;
use crate::GameState;

const COIN_SIZE: f32 = 30.0;

#[derive(Component)]
pub struct Coin {
    pub value: u32,
}

#[derive(Component)]
pub struct CoinAttracted {
    origin: Vec2,
    elapsed: f32,
}

pub fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        spawn_coins
            .in_set(PostGameSet)
            .after(death_system)
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (attract_coins, move_coins, collect_coins)
            .chain()
            .in_set(GameSet::WaveManagement),
    );
}

fn spawn_coins(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    wave_enemy_query: Query<&Transform, With<WaveEnemy>>,
    globals: Res<Globals>,
) {
    for event in death_events.read() {
        let Ok(transform) = wave_enemy_query.get(event.entity) else {
            continue;
        };
        let position = crate::coord::to_2d(transform.translation);

        commands
            .spawn((
                Name::new("Coin"),
                Coin {
                    value: globals.coins_per_kill,
                },
                Transform::from_translation(crate::coord::ground_pos(position))
                    .with_scale(Vec3::splat(COIN_SIZE)),
                Visibility::default(),
                CombatScoped,
            ))
            .with_children(|parent| {
                parent.spawn(Shadow);
                parent.spawn((
                    CircleShape {
                        color: palette::color("coin"),
                        flash_color: None,
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                ));
            });
    }
}

fn attract_coins(
    mut commands: Commands,
    player_query: Query<(&Transform, &ComputedStats), With<Player>>,
    coins: Query<(Entity, &Transform), (With<Coin>, Without<CoinAttracted>)>,
) {
    let Ok((player_transform, stats)) = player_query.single() else {
        return;
    };
    let player_pos = crate::coord::to_2d(player_transform.translation);
    let radius = {
        let r = stats.final_of(Stat::PickupRadius);
        if r > 0.0 { r } else { 2000.0 }
    };
    let radius_sq = radius * radius;

    for (entity, coin_transform) in &coins {
        let coin_pos = crate::coord::to_2d(coin_transform.translation);
        let dist_sq = player_pos.distance_squared(coin_pos);
        if dist_sq < radius_sq {
            commands.entity(entity).insert(CoinAttracted {
                origin: coin_pos,
                elapsed: 0.0,
            });
        }
    }
}

fn move_coins(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut coins: Query<(&mut Transform, &mut CoinAttracted), (With<Coin>, Without<Player>)>,
    globals: Res<Globals>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = crate::coord::to_2d(player_transform.translation);
    let dt = time.delta_secs();
    let duration = globals.coin_attraction_duration;

    for (mut transform, mut attracted) in &mut coins {
        attracted.elapsed += dt;
        let t = (attracted.elapsed / duration).min(1.0);
        let eased = t * t;
        let pos = attracted.origin.lerp(player_pos, eased);
        let ground = crate::coord::ground_pos(pos);
        transform.translation.x = ground.x;
        transform.translation.z = ground.z;
    }
}

fn collect_coins(
    mut commands: Commands,
    coins: Query<(Entity, &Coin, &CoinAttracted)>,
    mut money: ResMut<PlayerMoney>,
    globals: Res<Globals>,
) {
    let duration = globals.coin_attraction_duration;
    for (entity, coin, attracted) in &coins {
        if attracted.elapsed >= duration {
            money.earn(coin.value);
            commands.entity(entity).despawn();
        }
    }
}
