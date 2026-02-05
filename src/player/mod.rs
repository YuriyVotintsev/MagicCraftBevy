pub mod player_def;
pub mod selected_spells;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::GameState;
use crate::MovementLocked;
use crate::abilities::{AbilityInputs, AbilityRegistry, InputState, attach_ability};
use crate::physics::{ColliderShape, GameLayer};
use crate::schedule::GameSet;
use crate::schedule::PostGameSet;
use crate::stats::{
    ComputedStats, DeathEvent, DirtyStats, Health, Modifiers, StatCalculators, StatId, StatRegistry,
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
                (player_movement, player_defensive_input, player_active_input)
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
    ability_registry: Res<AbilityRegistry>,
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
            Health::new(max_life),
            AbilityInputs::new(),
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
        attach_ability(&mut commands, entity, Faction::Player, active_id, &ability_registry);
    }
    if let Some(passive_id) = selected_spells.passive {
        attach_ability(&mut commands, entity, Faction::Player, passive_id, &ability_registry);
    }
    if let Some(defensive_id) = selected_spells.defensive {
        attach_ability(&mut commands, entity, Faction::Player, defensive_id, &ability_registry);
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
    mut player_query: Query<(&Transform, &mut AbilityInputs), With<Player>>,
    selected_spells: Res<SelectedSpells>,
) {
    let Some(ability_id) = selected_spells.active else {
        return;
    };

    let Ok((player_transform, mut inputs)) = player_query.single_mut() else {
        return;
    };

    let pressed = mouse.pressed(MouseButton::Left);
    let just_pressed = mouse.just_pressed(MouseButton::Left);

    if !pressed && !just_pressed {
        inputs.inputs.remove(&ability_id);
        return;
    }

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

    inputs.set(ability_id, InputState {
        pressed,
        just_pressed,
        direction: direction.extend(0.0),
    });
}

fn player_defensive_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&LinearVelocity, &mut AbilityInputs), (With<Player>, Without<MovementLocked>)>,
    selected_spells: Res<SelectedSpells>,
) {
    let Some(ability_id) = selected_spells.defensive else {
        return;
    };

    let Ok((velocity, mut inputs)) = query.single_mut() else {
        return;
    };

    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let direction = velocity.0.normalize_or_zero();

    inputs.set(ability_id, InputState {
        pressed: true,
        just_pressed: true,
        direction: direction.extend(0.0),
    });
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
