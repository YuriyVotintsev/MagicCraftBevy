use bevy::prelude::*;

use crate::artifacts::{ArtifactRegistry, PlayerArtifacts};
use crate::game_state::GameState;
use crate::money::PlayerMoney;
use crate::wave::WavePhase;

#[derive(Resource, Default)]
pub struct SelectedArtifactSlot(pub Option<usize>);

#[derive(Component)]
pub struct ArtifactPanel;

#[derive(Component)]
pub struct ArtifactPanelSlot {
    pub slot: usize,
}

#[derive(Component)]
pub struct ArtifactPanelSellButton {
    pub slot: usize,
}

const NORMAL_SLOT: Color = Color::srgba(0.2, 0.2, 0.3, 0.8);
const HOVERED_SLOT: Color = Color::srgba(0.3, 0.3, 0.4, 0.9);
const SELECTED_SLOT: Color = Color::srgba(0.3, 0.3, 0.5, 0.9);
const SELL_BUTTON: Color = Color::srgba(0.6, 0.15, 0.15, 0.95);
const SELL_HOVERED: Color = Color::srgba(0.75, 0.2, 0.2, 0.95);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.15, 0.75);

pub fn spawn_artifact_panel(mut commands: Commands) {
    commands.init_resource::<SelectedArtifactSlot>();

    commands.spawn((
        Name::new("ArtifactPanel"),
        ArtifactPanel,
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            min_width: Val::Px(180.0),
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        GlobalZIndex(10),
    ));
}

pub fn rebuild_artifact_panel(
    mut commands: Commands,
    panel_query: Query<Entity, With<ArtifactPanel>>,
    artifacts: Res<PlayerArtifacts>,
    registry: Res<ArtifactRegistry>,
    selected: Res<SelectedArtifactSlot>,
    wave_phase: Option<Res<State<WavePhase>>>,
) {
    if !artifacts.is_changed() && !selected.is_changed() {
        if let Some(ref phase) = wave_phase {
            if !phase.is_changed() {
                return;
            }
        } else {
            return;
        }
    }

    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    commands.entity(panel_entity).despawn_children();

    let equipped = artifacts.equipped();
    if equipped.is_empty() {
        return;
    }

    let is_shop = wave_phase
        .as_ref()
        .map_or(false, |p| **p == WavePhase::Shop);

    for (slot, artifact_id) in &equipped {
        let Some(def) = registry.get(*artifact_id) else {
            continue;
        };

        let is_selected = selected.0 == Some(*slot);
        let sell_price = (def.price + 1) / 2;

        if is_shop {
            let mut row = commands.spawn((
                Button,
                ArtifactPanelSlot { slot: *slot },
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(if is_selected {
                    SELECTED_SLOT
                } else {
                    NORMAL_SLOT
                }),
            ));

            row.with_children(|parent| {
                if is_selected {
                    parent.spawn((
                        Button,
                        ArtifactPanelSellButton { slot: *slot },
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(SELL_BUTTON),
                        children![(
                            Text(format!("Sell {}g", sell_price)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(GOLD_COLOR),
                        )],
                    ));
                }

                parent.spawn((
                    Text::new(&def.name),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                ));
            });

            let row_id = row.id();
            commands.entity(panel_entity).add_child(row_id);
        } else {
            let row = commands
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    children![(
                        Text::new(&def.name),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    )],
                ))
                .id();
            commands.entity(panel_entity).add_child(row);
        }
    }
}

pub fn handle_artifact_slot_click(
    interaction_query: Query<(&Interaction, &ArtifactPanelSlot), Changed<Interaction>>,
    mut selected: ResMut<SelectedArtifactSlot>,
) {
    for (interaction, slot) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if selected.0 == Some(slot.slot) {
            selected.0 = None;
        } else {
            selected.0 = Some(slot.slot);
        }
    }
}

pub fn handle_panel_sell_click(
    interaction_query: Query<(&Interaction, &ArtifactPanelSellButton), Changed<Interaction>>,
    mut money: ResMut<PlayerMoney>,
    mut artifacts: ResMut<PlayerArtifacts>,
    mut selected: ResMut<SelectedArtifactSlot>,
    registry: Res<ArtifactRegistry>,
) {
    for (interaction, sell_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(artifact_id) = artifacts.sell(sell_btn.slot) {
            if let Some(def) = registry.get(artifact_id) {
                money.0 += (def.price + 1) / 2;
            }
        }
        selected.0 = None;
    }
}

pub fn update_panel_button_colors(
    mut slot_query: Query<
        (&Interaction, &mut BackgroundColor, &ArtifactPanelSlot),
        Without<ArtifactPanelSellButton>,
    >,
    mut sell_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<ArtifactPanelSellButton>, Without<ArtifactPanelSlot>),
    >,
    selected: Res<SelectedArtifactSlot>,
) {
    for (interaction, mut color, slot) in &mut slot_query {
        let is_selected = selected.0 == Some(slot.slot);
        match interaction {
            Interaction::Hovered => *color = HOVERED_SLOT.into(),
            Interaction::None => {
                *color = if is_selected {
                    SELECTED_SLOT
                } else {
                    NORMAL_SLOT
                }
                .into()
            }
            _ => {}
        }
    }
    for (interaction, mut color) in &mut sell_query {
        match interaction {
            Interaction::Hovered => *color = SELL_HOVERED.into(),
            Interaction::None => *color = SELL_BUTTON.into(),
            _ => {}
        }
    }
}

pub fn clear_artifact_selection(mut selected: ResMut<SelectedArtifactSlot>) {
    selected.0 = None;
}
