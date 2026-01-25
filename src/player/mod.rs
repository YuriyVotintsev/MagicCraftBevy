mod player_def;

use bevy::prelude::*;
use std::collections::HashMap;

use crate::abilities::{
    Abilities, AbilityDef, AbilityInput, AbilityRegistry, AbilityId,
    ActivatorDef, ActivatorRegistry, EffectDef, EffectRegistry, ParamValue,
};
use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::fsm::{Collider, ColliderShape};
use crate::stats::{
    ComputedStats, DirtyStats, Health, Modifiers, StatCalculators, StatId, StatRegistry,
};

use player_def::{load_player_def, PlayerDef};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerDefResource(pub PlayerDef);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let player_def = load_player_def("assets/player.ron");
        app.insert_resource(PlayerDefResource(player_def))
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, player_shooting, clamp_player));
    }
}

fn spawn_player(
    mut commands: Commands,
    player_def_res: Res<PlayerDefResource>,
    stat_registry: Res<StatRegistry>,
    calculators: Res<StatCalculators>,
    mut ability_registry: ResMut<AbilityRegistry>,
    activator_registry: Res<ActivatorRegistry>,
    mut effect_registry: ResMut<EffectRegistry>,
) {
    let player_def = &player_def_res.0;

    let mut modifiers = Modifiers::new();
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::new(stat_registry.len());

    for i in 0..stat_registry.len() {
        dirty.mark(StatId(i as u32));
    }

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

    let fireball_id = create_fireball_ability(
        &stat_registry,
        &mut ability_registry,
        &activator_registry,
        &mut effect_registry,
    );

    let mut abilities = Abilities::new();
    abilities.add(fireball_id);

    let collider = Collider {
        shape: ColliderShape::Circle,
        size: player_def.collider_size,
    };

    commands.spawn((
        Player,
        collider,
        modifiers,
        computed,
        dirty,
        Health::new(max_life),
        abilities,
        AbilityInput::new(),
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
    ));
}

fn create_fireball_ability(
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> AbilityId {
    let fireball_id = ability_registry.allocate_id("fireball");

    let activator_type = activator_registry.get_id("on_input").unwrap();

    let spawn_projectile_type = effect_registry.get_id("spawn_projectile").unwrap();
    let damage_type = effect_registry.get_id("damage").unwrap();

    let amount_param = effect_registry.get_or_insert_param_id("amount");
    let on_hit_param = effect_registry.get_or_insert_param_id("on_hit");

    let physical_damage_id = stat_registry.get("physical_damage").unwrap();

    let damage_effect = EffectDef {
        effect_type: damage_type,
        params: HashMap::from([(
            amount_param,
            ParamValue::Stat(physical_damage_id),
        )]),
    };

    let spawn_effect = EffectDef {
        effect_type: spawn_projectile_type,
        params: HashMap::from([(
            on_hit_param,
            ParamValue::EffectList(vec![damage_effect]),
        )]),
    };

    let ability_def = AbilityDef {
        id: fireball_id,
        tags: Vec::new(),
        activator: ActivatorDef {
            activator_type,
            params: HashMap::new(),
        },
        effects: vec![spawn_effect],
    };

    ability_registry.register(ability_def);

    fireball_id
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut Transform, &ComputedStats), With<Player>>,
) {
    let Ok((mut transform, stats)) = query.single_mut() else {
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

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        let speed = stat_registry
            .get("movement_speed")
            .map(|id| stats.get(id))
            .unwrap_or(400.0);
        transform.translation.x += direction.x * speed * time.delta_secs();
        transform.translation.y += direction.y * speed * time.delta_secs();
    }
}

fn player_shooting(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut player_query: Query<(&Transform, &mut AbilityInput), With<Player>>,
    ability_registry: Res<AbilityRegistry>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_transform, mut input)) = player_query.single_mut() else {
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

    if direction != Vec2::ZERO {
        if let Some(fireball_id) = ability_registry.get_id("fireball") {
            input.want_to_cast = Some(fireball_id);
            input.target_direction = Some(direction.extend(0.0));
            input.target_point = Some(world_pos.extend(0.0));
        }
    }
}

fn clamp_player(
    player_def_res: Res<PlayerDefResource>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let player_size = player_def_res.0.visual.size;
    let half_width = ARENA_WIDTH / 2.0 - player_size / 2.0;
    let half_height = ARENA_HEIGHT / 2.0 - player_size / 2.0;

    if let Ok(mut transform) = query.single_mut() {
        transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
        transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
    }
}
