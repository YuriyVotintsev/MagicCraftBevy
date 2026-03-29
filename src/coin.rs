use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::blueprints::components::common::shadow::Shadow;
use crate::blueprints::components::common::sprite::CircleSprite;
use crate::money::PlayerMoney;
use crate::palette;
use crate::player::Player;
use crate::schedule::{GameSet, PostGameSet};
use crate::stats::{death_system, ComputedStats, DeathEvent, StatRegistry};
use crate::wave::{WaveEnemy, WavePhase};
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

pub struct CoinPlugin;

impl Plugin for CoinPlugin {
    fn build(&self, app: &mut App) {
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
}

fn spawn_coins(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    wave_enemy_query: Query<&Transform, With<WaveEnemy>>,
    balance: Res<GameBalance>,
) {
    for event in death_events.read() {
        let Ok(transform) = wave_enemy_query.get(event.entity) else {
            continue;
        };
        let position = transform.translation.truncate();

        commands
            .spawn((
                Name::new("Coin"),
                Coin {
                    value: balance.run.coins_per_kill,
                },
                Transform::from_translation(position.extend(0.0))
                    .with_scale(Vec3::splat(COIN_SIZE)),
                Visibility::default(),
                DespawnOnExit(WavePhase::Combat),
            ))
            .with_children(|parent| {
                parent.spawn(Shadow {
                    y_offset: -0.42,
                    opacity: 0.45,
                });
                parent.spawn(CircleSprite {
                    color: palette::color("coin"),
                });
            });
    }
}

fn attract_coins(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    player_query: Query<(&Transform, &ComputedStats), With<Player>>,
    coins: Query<(Entity, &Transform), (With<Coin>, Without<CoinAttracted>)>,
) {
    let Ok((player_transform, stats)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let radius = stat_registry
        .get("pickup_radius")
        .map(|id| stats.get(id))
        .unwrap_or(2000.0);
    let radius_sq = radius * radius;

    for (entity, coin_transform) in &coins {
        let coin_pos = coin_transform.translation.truncate();
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
    balance: Res<GameBalance>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();
    let duration = balance.run.coin_attraction_duration;

    for (mut transform, mut attracted) in &mut coins {
        attracted.elapsed += dt;
        let t = (attracted.elapsed / duration).min(1.0);
        let eased = t * t;
        let pos = attracted.origin.lerp(player_pos, eased);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

fn collect_coins(
    mut commands: Commands,
    coins: Query<(Entity, &Coin, &CoinAttracted)>,
    mut money: ResMut<PlayerMoney>,
    balance: Res<GameBalance>,
) {
    let duration = balance.run.coin_attraction_duration;
    for (entity, coin, attracted) in &coins {
        if attracted.elapsed >= duration {
            money.earn(coin.value);
            commands.entity(entity).despawn();
        }
    }
}
