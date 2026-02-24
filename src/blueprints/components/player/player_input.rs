use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use serde::Deserialize;

use crate::arena::CameraYaw;
use crate::blueprints::{BlueprintActivationInput, SpawnSource};
use crate::blueprints::context::TargetInfo;
use crate::coords::vec3_to_2d;
use crate::player::{SelectedSpells, SpellSlot};
use crate::schedule::GameSet;
use crate::wave::WavePhase;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum InputTrigger {
    MouseHold(MouseButtonKind),
    KeyJustPressed(KeyKind),
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum MouseButtonKind {
    Left,
    Right,
    Middle,
}

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
pub enum KeyKind {
    Space,
}

impl From<KeyKind> for KeyCode {
    fn from(kind: KeyKind) -> Self {
        match kind {
            KeyKind::Space => KeyCode::Space,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum TargetingMode {
    Cursor,
    MovementDirection,
    Untargeted,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputBinding {
    pub slot: SpellSlot,
    pub trigger: InputTrigger,
    pub targeting: TargetingMode,
}

#[blueprint_component]
pub struct PlayerInput {
    pub bindings: Vec<InputBinding>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        player_input_system
            .in_set(GameSet::Input)
            .run_if(in_state(WavePhase::Combat)),
    );
}

fn player_input_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_yaw: Res<CameraYaw>,
    player_query: Query<(Entity, &LinearVelocity, &PlayerInput)>,
    selected_spells: Res<SelectedSpells>,
    mut activation_query: Query<(&SpawnSource, &mut BlueprintActivationInput)>,
) {
    for (player_entity, velocity, player_input) in &player_query {
        for binding in &player_input.bindings {
            let triggered = match binding.trigger {
                InputTrigger::MouseHold(btn) => mouse.pressed(btn.into()),
                InputTrigger::KeyJustPressed(key) => keyboard.just_pressed(key.into()),
                InputTrigger::Auto => true,
            };

            if !triggered {
                continue;
            }

            let Some(blueprint_id) = selected_spells.get(binding.slot) else {
                continue;
            };

            let target = match binding.targeting {
                TargetingMode::Cursor => {
                    let yaw = camera_yaw.0;
                    let direction = Vec2::new(-yaw.sin(), yaw.cos());
                    TargetInfo::from_direction(direction)
                }
                TargetingMode::MovementDirection => {
                    TargetInfo::from_direction(vec3_to_2d(velocity.0).normalize_or_zero())
                }
                TargetingMode::Untargeted => TargetInfo::EMPTY,
            };

            for (source, mut input) in &mut activation_query {
                if source.blueprint_id == blueprint_id && source.caster.entity == Some(player_entity) {
                    input.pressed = true;
                    input.target = target;
                }
            }
        }
    }
}
