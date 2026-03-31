use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use crate::arena::CameraAngle;
use crate::wave::CombatPhase;

const SLIDER_MIN: f32 = 1.0;
const SLIDER_MAX: f32 = 89.0;

#[derive(Component)]
pub(super) struct CameraAngleSlider;

#[derive(Component)]
pub(super) struct SliderFill;

#[derive(Component)]
pub(super) struct SliderValueText;

pub(super) fn toggle_dev_menu(
    key: Res<ButtonInput<KeyCode>>,
    combat_phase: Res<State<CombatPhase>>,
    mut next_phase: ResMut<NextState<CombatPhase>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if key.just_pressed(KeyCode::Backquote) {
        match combat_phase.get() {
            CombatPhase::Running => {
                virtual_time.pause();
                next_phase.set(CombatPhase::DevMenu);
            }
            CombatPhase::DevMenu => {
                virtual_time.unpause();
                next_phase.set(CombatPhase::Running);
            }
            CombatPhase::Paused => {
                next_phase.set(CombatPhase::DevMenu);
            }
        }
    }
    if key.just_pressed(KeyCode::Escape) && *combat_phase.get() == CombatPhase::DevMenu {
        virtual_time.unpause();
        next_phase.set(CombatPhase::Running);
    }
}

pub(super) fn spawn_dev_menu(mut commands: Commands, camera_angle: Res<CameraAngle>) {
    let t = (camera_angle.degrees - SLIDER_MIN) / (SLIDER_MAX - SLIDER_MIN);

    commands.spawn((
        Name::new("DevMenuRoot"),
        DespawnOnExit(CombatPhase::DevMenu),
        GlobalZIndex(100),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::all(Val::Px(40.0)),
                    width: Val::Px(500.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                children![
                    (
                        Text::new("Dev Menu"),
                        TextFont { font_size: 48.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.9)),
                        Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() }
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Camera Angle"),
                                TextFont { font_size: 20.0, ..default() },
                                TextColor(Color::srgb(0.7, 0.7, 0.7))
                            ),
                            (
                                SliderValueText,
                                Text::new(format!("{:.0}\u{00b0}", camera_angle.degrees)),
                                TextFont { font_size: 20.0, ..default() },
                                TextColor(Color::srgb(0.9, 0.9, 0.9))
                            )
                        ]
                    ),
                    (
                        CameraAngleSlider,
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                        children![
                            (
                                SliderFill,
                                Node {
                                    width: Val::Percent(t * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.6, 0.9)),
                            )
                        ]
                    )
                ]
            )
        ],
    ));
}

pub(super) fn slider_interaction(
    track_query: Query<(&Interaction, &UiGlobalTransform, &ComputedNode), With<CameraAngleSlider>>,
    mut fill_query: Query<&mut Node, With<SliderFill>>,
    mut text_query: Query<&mut Text, With<SliderValueText>>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_angle: ResMut<CameraAngle>,
) {
    for (interaction, ui_transform, computed) in &track_query {
        let active = *interaction == Interaction::Pressed
            || (*interaction == Interaction::Hovered && mouse.pressed(MouseButton::Left));
        if !active {
            continue;
        }
        let Ok(window) = windows.single() else { continue };
        let Some(cursor_pos) = window.cursor_position() else { continue };

        let Some(inverse) = ui_transform.try_inverse() else { continue };
        let local = inverse.transform_point2(cursor_pos * window.scale_factor());
        let node_size = computed.size();
        let t = ((local.x / node_size.x) + 0.5).clamp(0.0, 1.0);
        let value = SLIDER_MIN + t * (SLIDER_MAX - SLIDER_MIN);

        camera_angle.degrees = value;

        for mut node in &mut fill_query {
            node.width = Val::Percent(t * 100.0);
        }
        for mut text in &mut text_query {
            *text = Text::new(format!("{:.0}\u{00b0}", value));
        }
    }
}
