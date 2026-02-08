pub mod selected_spells;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::GameState;
use crate::blueprints::spawn_blueprint_entity;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::entity_def::EntityDef;
use crate::schedule::PostGameSet;
use crate::stats::{DeathEvent, death_system};
use crate::wave::WavePhase;

pub use selected_spells::{SelectedSpells, SpellSlot};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerEntityDef(pub EntityDef);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSpells>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnExit(WavePhase::Combat), reset_player_velocity)
            .add_systems(PostUpdate, handle_player_death.after(death_system).in_set(PostGameSet));
    }
}

fn reset_player_velocity(mut query: Query<&mut LinearVelocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 = Vec2::ZERO;
    }
}

fn spawn_player(
    mut spawner: EntitySpawner,
    player_def: Res<PlayerEntityDef>,
    selected_spells: Res<SelectedSpells>,
) {
    let entity = spawner.spawn_root(&player_def.0, Faction::Player);
    spawner.commands.entity(entity).insert((
        Name::new("Player"),
        Player,
        DespawnOnExit(GameState::Playing),
    ));

    if let Some(active_id) = selected_spells.active {
        spawn_blueprint_entity(&mut spawner.commands, entity, Faction::Player, active_id, false);
    }
    if let Some(passive_id) = selected_spells.passive {
        spawn_blueprint_entity(&mut spawner.commands, entity, Faction::Player, passive_id, false);
    }
    if let Some(defensive_id) = selected_spells.defensive {
        spawn_blueprint_entity(&mut spawner.commands, entity, Faction::Player, defensive_id, false);
    }
}

fn handle_player_death(
    mut death_events: MessageReader<DeathEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<(), With<Player>>,
) {
    for event in death_events.read() {
        if player_query.contains(event.entity) {
            next_state.set(GameState::GameOver);
        }
    }
}
