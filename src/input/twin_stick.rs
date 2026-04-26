// Twin-stick touch controller used as the wasm backend for `PlayerInputPlugin`.
//
// Coordinate-space note:
//   * `UiGlobalTransform.translation` / `ComputedNode.size` are in physical pixels.
//   * `Touches::position()` is in CSS pixels (winit calls `to_logical(dpr)`).
//   * `Val::Px(v)` renders as `v * (dpr * ui_scale)` physical pixels, i.e. 1 Val::Px
//     unit equals `1/ui_scale` CSS pixels.
// We work in Val::Px units throughout: convert touch positions from CSS to Val::Px
// by dividing by ui_scale, use `factor = inverse_scale_factor` to bring ComputedNode
// sizes/translations into the same space, and write `Val::Px(...)` back without any
// further division. Mixing CSS math with raw Val::Px writes is what breaks the
// upstream `virtual_joystick` crate when `ui_scale != 1`.

use bevy::prelude::*;

use super::PlayerIntent;
use crate::game_state::GameState;
use crate::schedule::GameSet;
use crate::wave::CombatPhase;

const STICK_RADIUS_PX: f32 = 80.0;
const KNOB_SIZE_PX: f32 = 60.0;
const OUTLINE_SIZE_PX: f32 = 160.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum StickId {
    Move,
    Aim,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PointerId {
    Touch(u64),
    Mouse,
}

#[derive(Component)]
struct TouchStick {
    id: StickId,
    pointer: Option<PointerId>,
    start_local: Vec2,
    current_local: Vec2,
}

#[derive(Component)]
struct StickOutline {
    stick: Entity,
}

#[derive(Component)]
struct StickKnob {
    stick: Entity,
}

pub fn build(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), spawn_sticks)
        .add_systems(OnExit(GameState::Playing), reset_intent)
        .add_systems(
            Update,
            (update_sticks, write_intent, update_visuals)
                .chain()
                .before(GameSet::Input)
                .run_if(in_state(CombatPhase::Running)),
        );
}

fn reset_intent(mut intent: ResMut<PlayerIntent>) {
    *intent = PlayerIntent::default();
}

fn spawn_sticks(mut commands: Commands) {
    spawn_stick(&mut commands, StickId::Move, true);
    spawn_stick(&mut commands, StickId::Aim, false);
}

fn spawn_stick(commands: &mut Commands, id: StickId, left_side: bool) {
    let (left, right) = if left_side {
        (Val::Px(0.0), Val::Auto)
    } else {
        (Val::Auto, Val::Px(0.0))
    };
    let (outline_color, zone_tint) = if left_side {
        (
            Color::srgba(0.3, 0.6, 1.0, 0.35),
            Color::srgba(0.3, 0.6, 1.0, 0.05),
        )
    } else {
        (
            Color::srgba(1.0, 0.5, 0.15, 0.35),
            Color::srgba(1.0, 0.5, 0.15, 0.05),
        )
    };

    let stick = commands
        .spawn((
            Name::new(format!("TouchStick::{:?}", id)),
            DespawnOnExit(GameState::Playing),
            TouchStick {
                id,
                pointer: None,
                start_local: Vec2::ZERO,
                current_local: Vec2::ZERO,
            },
            Node {
                width: Val::Percent(45.0),
                height: Val::Percent(60.0),
                position_type: PositionType::Absolute,
                left,
                right,
                bottom: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(zone_tint),
            GlobalZIndex(30),
        ))
        .id();

    commands.spawn((
        StickOutline { stick },
        ChildOf(stick),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(OUTLINE_SIZE_PX),
            height: Val::Px(OUTLINE_SIZE_PX),
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            border_radius: BorderRadius::all(Val::Px(OUTLINE_SIZE_PX / 2.0)),
            ..default()
        },
        BackgroundColor(outline_color),
        Visibility::Hidden,
    ));

    commands.spawn((
        StickKnob { stick },
        ChildOf(stick),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(KNOB_SIZE_PX),
            height: Val::Px(KNOB_SIZE_PX),
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            border_radius: BorderRadius::all(Val::Px(KNOB_SIZE_PX / 2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Visibility::Hidden,
    ));
}

fn zone_rect_px_units(node: &ComputedNode, ui_transform: &UiGlobalTransform) -> Rect {
    let factor = node.inverse_scale_factor;
    let center = ui_transform.translation * factor;
    let size = node.size() * factor;
    Rect::from_center_size(center, size)
}

fn update_sticks(
    mut sticks: Query<(&mut TouchStick, &ComputedNode, &UiGlobalTransform)>,
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    ui_scale: Res<UiScale>,
) {
    let scale = ui_scale.0.max(1e-3);
    let cursor_css = windows.single().ok().and_then(|w| w.cursor_position());
    let cursor_px = cursor_css.map(|p| p / scale);
    for (mut stick, node, ui_transform) in &mut sticks {
        let rect = zone_rect_px_units(node, ui_transform);
        let top_left = rect.min;

        if let Some(pid) = stick.pointer {
            match pid {
                PointerId::Touch(id) => {
                    if let Some(touch) = touches.get_pressed(id) {
                        let pos_px = touch.position() / scale;
                        stick.current_local = pos_px - top_left;
                    } else {
                        stick.pointer = None;
                    }
                }
                PointerId::Mouse => {
                    if mouse.pressed(MouseButton::Left) {
                        if let Some(pos) = cursor_px {
                            stick.current_local = pos - top_left;
                        }
                    } else {
                        stick.pointer = None;
                    }
                }
            }
            continue;
        }

        let mut claimed = false;
        for touch in touches.iter() {
            if !touches.just_pressed(touch.id()) {
                continue;
            }
            let pos_px = touch.position() / scale;
            if rect.contains(pos_px) {
                let local = pos_px - top_left;
                stick.pointer = Some(PointerId::Touch(touch.id()));
                stick.start_local = local;
                stick.current_local = local;
                claimed = true;
                break;
            }
        }
        if claimed {
            continue;
        }
        if mouse.just_pressed(MouseButton::Left) {
            if let Some(pos) = cursor_px {
                if rect.contains(pos) {
                    let local = pos - top_left;
                    stick.pointer = Some(PointerId::Mouse);
                    stick.start_local = local;
                    stick.current_local = local;
                }
            }
        }
    }
}

fn stick_axis(stick: &TouchStick) -> Vec2 {
    if stick.pointer.is_none() {
        return Vec2::ZERO;
    }
    let delta = stick.current_local - stick.start_local;
    let len = delta.length();
    if len < 1e-3 {
        return Vec2::ZERO;
    }
    let mag = (len / STICK_RADIUS_PX).min(1.0);
    let dir = delta / len;
    // Bevy UI Y grows downward; gameplay Y grows upward.
    Vec2::new(dir.x, -dir.y) * mag
}

fn write_intent(sticks: Query<&TouchStick>, mut intent: ResMut<PlayerIntent>) {
    for stick in &sticks {
        let axis = stick_axis(stick);
        match stick.id {
            StickId::Move => intent.move_dir = axis,
            StickId::Aim => {
                intent.aim_dir = axis.normalize_or_zero();
                intent.fire = axis.length_squared() > 0.0;
            }
        }
    }
}

fn update_visuals(
    sticks: Query<&TouchStick>,
    mut outline_query: Query<
        (&StickOutline, &mut Node, &mut Visibility),
        (Without<StickKnob>, Without<TouchStick>),
    >,
    mut knob_query: Query<
        (&StickKnob, &mut Node, &mut Visibility),
        (Without<StickOutline>, Without<TouchStick>),
    >,
) {
    for (outline, mut node, mut vis) in &mut outline_query {
        let Ok(stick) = sticks.get(outline.stick) else {
            continue;
        };
        if stick.pointer.is_none() {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;
        let tl = stick.start_local - Vec2::splat(OUTLINE_SIZE_PX / 2.0);
        node.left = Val::Px(tl.x);
        node.top = Val::Px(tl.y);
    }
    for (knob, mut node, mut vis) in &mut knob_query {
        let Ok(stick) = sticks.get(knob.stick) else {
            continue;
        };
        if stick.pointer.is_none() {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;
        let delta = stick.current_local - stick.start_local;
        let d = delta.length();
        let clamped = if d > STICK_RADIUS_PX {
            delta * (STICK_RADIUS_PX / d)
        } else {
            delta
        };
        let tl = stick.start_local + clamped - Vec2::splat(KNOB_SIZE_PX / 2.0);
        node.left = Val::Px(tl.x);
        node.top = Val::Px(tl.y);
    }
}
