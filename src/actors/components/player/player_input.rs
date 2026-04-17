use bevy::prelude::*;

use super::super::super::player::{fire_fireball, FIREBALL_COOLDOWN};
use crate::Faction;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::wave::WavePhase;

#[derive(Debug, Clone, Copy)]
pub enum InputTrigger {
    MouseHold(MouseButtonKind),
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButtonKind { Left, Right, Middle }

impl From<MouseButtonKind> for MouseButton {
    fn from(kind: MouseButtonKind) -> Self {
        match kind {
            MouseButtonKind::Left => MouseButton::Left,
            MouseButtonKind::Right => MouseButton::Right,
            MouseButtonKind::Middle => MouseButton::Middle,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TargetingMode { Cursor }

#[derive(Component)]
pub struct PlayerInput {
    pub trigger: InputTrigger,
    pub targeting: TargetingMode,
}

#[derive(Component, Default)]
pub struct PlayerAbilityCooldowns {
    pub current: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (tick_cooldowns, player_input_system)
            .chain()
            .in_set(GameSet::Input)
            .run_if(in_state(WavePhase::Combat)),
    );
}

fn tick_cooldowns(time: Res<Time>, mut q: Query<&mut PlayerAbilityCooldowns>) {
    let dt = time.delta_secs();
    for mut c in &mut q {
        c.current = (c.current - dt).max(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn player_input_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut player_query: Query<(Entity, &Transform, &PlayerInput, Option<&ComputedStats>, &mut PlayerAbilityCooldowns)>,
) {
    for (player_entity, player_transform, player_input, stats, mut cooldowns) in &mut player_query {
        let triggered = match player_input.trigger {
            InputTrigger::MouseHold(btn) => mouse.pressed(btn.into()),
        };
        if !triggered { continue }
        if cooldowns.current > 0.0 { continue }

        let direction = match player_input.targeting {
            TargetingMode::Cursor => {
                let Ok(window) = windows.single() else { continue };
                let Ok((camera, camera_transform)) = camera_query.single() else { continue };
                let Some(cursor_pos) = window.cursor_position() else { continue };
                let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { continue };
                let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { continue };
                let world_pos = ray.get_point(distance);
                let direction = crate::coord::to_2d(world_pos - player_transform.translation).normalize_or_zero();
                if direction == Vec2::ZERO { continue }
                direction
            }
        };

        let caster_pos = crate::coord::to_2d(player_transform.translation);
        fire_fireball(&mut commands, player_entity, caster_pos, Faction::Player, direction, stats);
        let attack_speed = stats
            .map(|s| s.final_of(Stat::AttackSpeed))
            .unwrap_or(1.0)
            .max(0.01);
        cooldowns.current = FIREBALL_COOLDOWN / attack_speed;
    }
}
