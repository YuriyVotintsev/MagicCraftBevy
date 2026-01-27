use bevy::prelude::*;

use crate::money::PlayerMoney;
use crate::schedule::{GameSet, PostGameSet};
use crate::stats::DeathEvent;
use crate::Faction;
use crate::GameState;

const BASE_ENEMIES: u32 = 5;
const ENEMIES_PER_WAVE: u32 = 3;
const WAVE_REWARD: u32 = 10;
const SHOP_DELAY: f32 = 2.0;

#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
pub enum WavePhase {
    #[default]
    Combat,
    ShopDelay,
    Shop,
}

#[derive(Resource)]
pub struct WaveState {
    pub current_wave: u32,
    pub spawned_count: u32,
    pub killed_count: u32,
    pub target_count: u32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            current_wave: 1,
            spawned_count: 0,
            killed_count: 0,
            target_count: BASE_ENEMIES,
        }
    }
}

impl WaveState {
    fn calculate_target(wave: u32) -> u32 {
        BASE_ENEMIES + (wave - 1) * ENEMIES_PER_WAVE
    }
}

#[derive(Component)]
pub struct WaveEnemy;

#[derive(Component)]
pub struct InvulnerableStack(pub u32);

impl InvulnerableStack {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn decrement(&mut self) -> bool {
        self.0 = self.0.saturating_sub(1);
        self.0 == 0
    }
}

pub fn add_invulnerability(commands: &mut Commands, entity: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mut stack) = world.get_mut::<InvulnerableStack>(entity) {
            stack.increment();
        } else if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(InvulnerableStack(1));
        }
    });
}

pub fn remove_invulnerability(commands: &mut Commands, entity: Entity) {
    commands.queue(move |world: &mut World| {
        let should_remove = {
            if let Some(mut stack) = world.get_mut::<InvulnerableStack>(entity) {
                stack.decrement()
            } else {
                false
            }
        };
        if should_remove {
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.remove::<InvulnerableStack>();
            }
        }
    });
}

#[derive(Resource)]
pub struct ShopDelayTimer(pub Timer);

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveState>()
            .init_resource::<PlayerMoney>()
            .insert_resource(ShopDelayTimer(Timer::from_seconds(
                SHOP_DELAY,
                TimerMode::Once,
            )))
            .add_systems(OnEnter(GameState::Playing), reset_wave_state)
            .add_systems(
                Update,
                check_wave_completion
                    .in_set(GameSet::WaveManagement)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                PostUpdate,
                track_wave_kills
                    .in_set(PostGameSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                shop_delay_tick.run_if(in_state(WavePhase::ShopDelay)),
            )
            .add_systems(OnExit(WavePhase::Shop), start_next_wave);
    }
}

fn reset_wave_state(
    mut wave_state: ResMut<WaveState>,
    mut money: ResMut<PlayerMoney>,
    mut shop_timer: ResMut<ShopDelayTimer>,
) {
    *wave_state = WaveState::default();
    money.0 = 0;
    shop_timer.0.reset();
}

fn track_wave_kills(
    mut death_events: MessageReader<DeathEvent>,
    mut wave_state: ResMut<WaveState>,
    wave_enemy_query: Query<(), With<WaveEnemy>>,
) {
    for event in death_events.read() {
        if wave_enemy_query.contains(event.entity) {
            wave_state.killed_count += 1;
        }
    }
}

fn check_wave_completion(
    mut commands: Commands,
    wave_state: Res<WaveState>,
    enemies_query: Query<&Faction>,
    player_query: Query<Entity, With<crate::player::Player>>,
    mut next_phase: ResMut<NextState<WavePhase>>,
    mut money: ResMut<PlayerMoney>,
    mut shop_timer: ResMut<ShopDelayTimer>,
) {
    let all_spawned = wave_state.spawned_count >= wave_state.target_count;
    let all_wave_enemies_killed = wave_state.killed_count >= wave_state.spawned_count;
    let no_enemies_alive = !enemies_query.iter().any(|f| *f == Faction::Enemy);

    if all_spawned && all_wave_enemies_killed && no_enemies_alive && wave_state.spawned_count > 0 {
        money.0 += WAVE_REWARD;
        shop_timer.0.reset();

        if let Ok(player_entity) = player_query.single() {
            add_invulnerability(&mut commands, player_entity);
        }

        next_phase.set(WavePhase::ShopDelay);
    }
}

fn shop_delay_tick(
    time: Res<Time>,
    mut timer: ResMut<ShopDelayTimer>,
    mut next_phase: ResMut<NextState<WavePhase>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next_phase.set(WavePhase::Shop);
    }
}

fn start_next_wave(
    mut commands: Commands,
    mut wave_state: ResMut<WaveState>,
    player_query: Query<Entity, With<crate::player::Player>>,
) {
    wave_state.current_wave += 1;
    wave_state.spawned_count = 0;
    wave_state.killed_count = 0;
    wave_state.target_count = WaveState::calculate_target(wave_state.current_wave);

    if let Ok(player_entity) = player_query.single() {
        remove_invulnerability(&mut commands, player_entity);
    }
}
