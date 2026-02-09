pub mod hero_class;
pub mod selected_spells;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::GameState;
use crate::blueprints::{BlueprintRegistry, spawn_blueprint_entity};
use crate::blueprints::components::ComponentDef;
use crate::blueprints::spawn::EntitySpawner;
use crate::schedule::PostGameSet;
use crate::stats::{DeathEvent, death_system};
use crate::wave::WavePhase;

pub use hero_class::{AvailableHeroes, SelectedHero};
pub use selected_spells::{SelectedSpells, SpellSlot};

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSpells>()
            .init_resource::<SelectedHero>()
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
    selected_hero: Res<SelectedHero>,
    available_heroes: Res<AvailableHeroes>,
    blueprint_registry: Res<BlueprintRegistry>,
    selected_spells: Res<SelectedSpells>,
) {
    let Some(class_index) = selected_hero.0 else {
        warn!("No hero selected, cannot spawn player");
        return;
    };
    let Some(class) = available_heroes.classes.get(class_index) else {
        warn!("Hero class index out of bounds: {}", class_index);
        return;
    };
    let Some(blueprint_def) = blueprint_registry.get(available_heroes.base_blueprint) else {
        warn!("Base hero blueprint not found");
        return;
    };
    let Some(base_entity_def) = blueprint_def.entities.first() else {
        warn!("Base hero blueprint has no entities");
        return;
    };

    let mut entity_def = base_entity_def.clone();
    for comp in &mut entity_def.components {
        if let ComponentDef::Sprite(ref mut def) = comp {
            def.color = class.color;
        }
    }

    let modifier_tuples: Vec<_> = class.modifiers.iter().map(|m| (m.stat, m.value)).collect();
    let entity = spawner.spawn_root(&entity_def, Faction::Player, &modifier_tuples);
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
