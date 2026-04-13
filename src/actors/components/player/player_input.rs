use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::actors::abilities::{fire_ability, AbilitiesBalance};
use crate::actors::TargetInfo;
use crate::Faction;
use crate::player::{SelectedSpells, SpellSlot};
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::wave::WavePhase;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum InputTrigger {
    MouseHold(MouseButtonKind),
    KeyJustPressed(KeyKind),
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize)]
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

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum KeyKind { Space }

impl From<KeyKind> for KeyCode {
    fn from(kind: KeyKind) -> Self {
        match kind { KeyKind::Space => KeyCode::Space }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum TargetingMode { Cursor, MovementDirection, Untargeted }

#[derive(Debug, Clone, Deserialize)]
pub struct InputBinding {
    pub slot: SpellSlot,
    pub trigger: InputTrigger,
    pub targeting: TargetingMode,
}

#[derive(Component)]
pub struct PlayerInput {
    pub bindings: Vec<InputBinding>,
}

#[derive(Component, Default)]
pub struct PlayerAbilityCooldowns {
    pub active: f32,
    pub passive: f32,
    pub defensive: f32,
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
        c.active = (c.active - dt).max(0.0);
        c.passive = (c.passive - dt).max(0.0);
        c.defensive = (c.defensive - dt).max(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn player_input_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut player_query: Query<(Entity, &Transform, &LinearVelocity, &PlayerInput, Option<&ComputedStats>, &mut PlayerAbilityCooldowns)>,
    selected_spells: Res<SelectedSpells>,
    abilities_balance: Res<AbilitiesBalance>,
) {
    for (player_entity, player_transform, velocity, player_input, stats, mut cooldowns) in &mut player_query {
        for binding in &player_input.bindings {
            let triggered = match binding.trigger {
                InputTrigger::MouseHold(btn) => mouse.pressed(btn.into()),
                InputTrigger::KeyJustPressed(key) => keyboard.just_pressed(key.into()),
                InputTrigger::Auto => true,
            };
            if !triggered { continue }

            let Some(kind) = selected_spells.get(binding.slot) else { continue };

            let slot_cd = match binding.slot {
                SpellSlot::Active => &mut cooldowns.active,
                SpellSlot::Passive => &mut cooldowns.passive,
                SpellSlot::Defensive => &mut cooldowns.defensive,
            };
            if *slot_cd > 0.0 { continue }

            let target = match binding.targeting {
                TargetingMode::Cursor => {
                    let Ok(window) = windows.single() else { continue };
                    let Ok((camera, camera_transform)) = camera_query.single() else { continue };
                    let Some(cursor_pos) = window.cursor_position() else { continue };
                    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { continue };
                    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { continue };
                    let world_pos = ray.get_point(distance);
                    let direction = crate::coord::to_2d(world_pos - player_transform.translation).normalize_or_zero();
                    if direction == Vec2::ZERO { continue }
                    TargetInfo::from_direction(direction)
                }
                TargetingMode::MovementDirection => {
                    TargetInfo::from_direction(crate::coord::to_2d(velocity.0).normalize_or_zero())
                }
                TargetingMode::Untargeted => TargetInfo::EMPTY,
            };

            let caster_pos = crate::coord::to_2d(player_transform.translation);
            let ability_cooldown = match kind {
                crate::actors::abilities::AbilityKind::Fireball => abilities_balance.fireball.cooldown,
                _ => 0.5,
            };
            fire_ability(&mut commands, kind, player_entity, caster_pos, Faction::Player, target, &abilities_balance, stats);
            *slot_cd = ability_cooldown;
        }
    }
}
