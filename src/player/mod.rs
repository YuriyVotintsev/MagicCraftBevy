pub mod player_def;
pub mod selected_spells;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::GameState;
use crate::MovementLocked;
use crate::blueprints::{BlueprintActivationInput, SpawnSource, spawn_blueprint_entity};
use crate::blueprints::context::TargetInfo;
use crate::physics::{ColliderShape, GameLayer};
use crate::schedule::GameSet;
use crate::schedule::PostGameSet;
use crate::blueprints::components::health::Health;
use crate::stats::{
    ComputedStats, DeathEvent, DirtyStats, Modifiers, StatCalculators, StatId, StatRegistry,
    death_system,
};
use crate::wave::WavePhase;

use player_def::PlayerDef;
pub use selected_spells::{SelectedSpells, SpellSlot};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerDefResource(pub PlayerDef);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSpells>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnExit(WavePhase::Combat), reset_player_velocity)
            .add_systems(
                Update,
                (player_movement, player_defensive_input, player_active_input, passive_auto_input)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(PostUpdate, handle_player_death.after(death_system).in_set(PostGameSet));
    }
}

fn reset_player_velocity(mut query: Query<&mut LinearVelocity, With<Player>>) {
    for mut velocity in &mut query {
        velocity.0 = Vec2::ZERO;
    }
}

fn spawn_player(
    mut commands: Commands,
    player_def_res: Res<PlayerDefResource>,
    stat_registry: Res<StatRegistry>,
    calculators: Res<StatCalculators>,
    selected_spells: Res<SelectedSpells>,
) {
    let player_def = &player_def_res.0;

    let mut modifiers = Modifiers::new();
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::new(stat_registry.len());

    dirty.mark_all((0..stat_registry.len() as u32).map(StatId));

    for (stat_name, value) in &player_def.base_stats {
        if let Some(stat_id) = stat_registry.get(stat_name) {
            modifiers.add(stat_id, *value, None);
        }
    }

    calculators.recalculate(&modifiers, &mut computed, &mut dirty);

    let max_life = stat_registry
        .get("max_life")
        .map(|id| computed.get(id))
        .unwrap_or(100.0);

    let collider = match player_def.collider.shape {
        ColliderShape::Circle => Collider::circle(player_def.collider.size / 2.0),
        ColliderShape::Rectangle => {
            Collider::rectangle(player_def.collider.size, player_def.collider.size)
        }
    };

    let player_layers = CollisionLayers::new(
        GameLayer::Player,
        [GameLayer::Enemy, GameLayer::EnemyProjectile, GameLayer::Wall],
    );

    let entity = commands.spawn((
        (
            Name::new("Player"),
            DespawnOnExit(GameState::Playing),
            Player,
            Faction::Player,
            collider,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::ZERO,
            player_layers,
        ),
        (
            modifiers,
            computed,
            dirty,
            Health { current: max_life },
            Sprite {
                color: Color::srgb(
                    player_def.visual.color[0],
                    player_def.visual.color[1],
                    player_def.visual.color[2],
                ),
                custom_size: Some(Vec2::splat(player_def.visual.size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ),
    )).id();

    if let Some(active_id) = selected_spells.active {
        spawn_blueprint_entity(&mut commands, entity, Faction::Player, active_id, false);
    }
    if let Some(passive_id) = selected_spells.passive {
        spawn_blueprint_entity(&mut commands, entity, Faction::Player, passive_id, false);
    }
    if let Some(defensive_id) = selected_spells.defensive {
        spawn_blueprint_entity(&mut commands, entity, Faction::Player, defensive_id, false);
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut LinearVelocity, &ComputedStats), (With<Player>, Without<MovementLocked>)>,
) {
    let Ok((mut velocity, stats)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let speed = stat_registry
        .get("movement_speed")
        .map(|id| stats.get(id))
        .unwrap_or(400.0);

    velocity.0 = if direction != Vec2::ZERO {
        direction.normalize() * speed
    } else {
        Vec2::ZERO
    };
}

fn player_active_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    selected_spells: Res<SelectedSpells>,
    mut blueprint_query: Query<(&SpawnSource, &mut BlueprintActivationInput)>,
) {
    let Some(ability_id) = selected_spells.active else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let direction = (world_pos - player_pos).normalize_or_zero();
    if direction == Vec2::ZERO {
        return;
    }

    for (source, mut input) in &mut blueprint_query {
        if source.blueprint_id == ability_id && source.caster.entity == Some(player_entity) {
            input.pressed = true;
            input.target = TargetInfo::from_direction(direction);
        }
    }
}

fn player_defensive_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<(Entity, &LinearVelocity), (With<Player>, Without<MovementLocked>)>,
    selected_spells: Res<SelectedSpells>,
    mut blueprint_query: Query<(&SpawnSource, &mut BlueprintActivationInput)>,
) {
    let Some(ability_id) = selected_spells.defensive else {
        return;
    };

    let Ok((player_entity, velocity)) = player_query.single() else {
        return;
    };

    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let direction = velocity.0.normalize_or_zero();

    for (source, mut input) in &mut blueprint_query {
        if source.blueprint_id == ability_id && source.caster.entity == Some(player_entity) {
            input.pressed = true;
            input.target = TargetInfo::from_direction(direction);
        }
    }
}

fn passive_auto_input(
    selected_spells: Res<SelectedSpells>,
    player_query: Query<Entity, With<Player>>,
    mut blueprint_query: Query<(&SpawnSource, &mut BlueprintActivationInput)>,
) {
    let Some(ability_id) = selected_spells.passive else {
        return;
    };
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for (source, mut input) in &mut blueprint_query {
        if source.blueprint_id == ability_id && source.caster.entity == Some(player_entity) {
            input.pressed = true;
        }
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
