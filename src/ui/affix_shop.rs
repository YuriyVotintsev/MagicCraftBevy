use bevy::prelude::*;

use crate::affixes::{
    apply_alteration, apply_augmentation, apply_chaos, sync_affix_modifiers, Affix, AffixRegistry,
    Affixes, OrbBehavior, OrbFlowState, OrbId, OrbRegistry, SlotOwner, SpellSlotTag,
};
use crate::blueprints::BlueprintRegistry;
use crate::money::PlayerMoney;
use crate::player::{Player, SelectedSpells, SpellSlot};
use crate::stats::{DirtyStats, Modifiers};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const DISABLED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const GREEN_COLOR: Color = Color::srgb(0.2, 0.9, 0.2);
const RED_COLOR: Color = Color::srgb(0.9, 0.2, 0.2);
const POPUP_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);

#[derive(Component)]
pub struct BuyOrbButton(pub OrbId);

#[derive(Component)]
pub struct OrbTooltipTarget(pub OrbId);

#[derive(Component)]
pub struct OrbTooltip;

#[derive(Component)]
pub struct OrbSection;

#[derive(Component)]
pub struct OrbPopup;

#[derive(Component)]
pub struct SlotSelectButton(pub Entity);

#[derive(Component)]
pub struct AcceptOrbButton;

#[derive(Component)]
pub struct CancelOrbButton;

pub fn build_orb_section(
    commands: &mut Commands,
    section: Entity,
    orb_registry: &OrbRegistry,
    money: u32,
    flow_state: &OrbFlowState,
) {
    let header = commands
        .spawn((
            Text::new("ORBS"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(section).add_child(header);

    let flow_busy = !matches!(flow_state, OrbFlowState::None);

    for orb_id in orb_registry.all_ids() {
        let Some(def) = orb_registry.get(orb_id) else {
            continue;
        };
        let can_buy = money >= def.price && !flow_busy;

        let row = commands
            .spawn((
                Interaction::default(),
                OrbTooltipTarget(orb_id),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text(format!("{} - {}g", def.name, def.price)),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::right(Val::Px(12.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn((
                        Button,
                        BuyOrbButton(orb_id),
                        OrbTooltipTarget(orb_id),
                        Node {
                            width: Val::Px(70.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if can_buy {
                            NORMAL_BUTTON
                        } else {
                            DISABLED_BUTTON
                        }),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Buy"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(if can_buy {
                                TEXT_COLOR
                            } else {
                                Color::srgb(0.4, 0.4, 0.4)
                            }),
                        ));
                    });
            })
            .id();
        commands.entity(section).add_child(row);
    }
}

fn spawn_slot_select_popup(
    commands: &mut Commands,
    slot_query: &Query<(Entity, &SpellSlotTag)>,
    selected_spells: &SelectedSpells,
    blueprint_registry: &BlueprintRegistry,
) {
    let popup = commands
        .spawn((
            OrbPopup,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(POPUP_BG),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.22, 0.98)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Select Slot"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(GOLD_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    for (slot_entity, slot_tag) in slot_query {
                        let spell_name = get_spell_name(
                            slot_tag.0,
                            selected_spells,
                            blueprint_registry,
                        );
                        let label = format!("{} - {}", slot_tag.0.label(), spell_name);

                        panel
                            .spawn((
                                Button,
                                SlotSelectButton(slot_entity),
                                Node {
                                    width: Val::Px(350.0),
                                    height: Val::Px(45.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::bottom(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextColor(TEXT_COLOR),
                                ));
                            });
                    }

                    panel
                        .spawn((
                            Button,
                            CancelOrbButton,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(12.0)),
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Cancel"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        })
        .id();
    let _ = popup;
}

fn spawn_preview_popup(
    commands: &mut Commands,
    original: &Affixes,
    preview: &Affixes,
    slot_type: SpellSlot,
    orb_behavior: OrbBehavior,
    affix_registry: &AffixRegistry,
    selected_spells: &SelectedSpells,
    blueprint_registry: &BlueprintRegistry,
) {
    let spell_name = get_spell_name(slot_type, selected_spells, blueprint_registry);

    let title = commands
        .spawn((
            Text(format!("{} - {}", slot_type.label(), spell_name)),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(24.0)),
                width: Val::Px(420.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.22, 0.98)),
        ))
        .id();

    commands.entity(panel).add_child(title);

    for i in 0..6 {
        let old = &original.affixes[i];
        let new = &preview.affixes[i];
        build_affix_diff_row(commands, panel, old, new, orb_behavior, affix_registry);
    }

    let accept_text = commands
        .spawn((
            Text::new("Accept"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ))
        .id();
    let accept_btn = commands
        .spawn((
            Button,
            AcceptOrbButton,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.4, 0.1)),
        ))
        .id();
    commands.entity(accept_btn).add_child(accept_text);

    let cancel_text = commands
        .spawn((
            Text::new("Cancel"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ))
        .id();
    let cancel_btn = commands
        .spawn((
            Button,
            CancelOrbButton,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.4, 0.1, 0.1)),
        ))
        .id();
    commands.entity(cancel_btn).add_child(cancel_text);

    let button_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            margin: UiRect::top(Val::Px(16.0)),
            column_gap: Val::Px(16.0),
            ..default()
        })
        .id();
    commands
        .entity(button_row)
        .add_children(&[accept_btn, cancel_btn]);
    commands.entity(panel).add_child(button_row);

    commands
        .spawn((
            OrbPopup,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(POPUP_BG),
            GlobalZIndex(100),
        ))
        .add_children(&[panel]);
}

fn build_affix_diff_row(
    commands: &mut Commands,
    parent_entity: Entity,
    old: &Option<Affix>,
    new: &Option<Affix>,
    orb_behavior: OrbBehavior,
    registry: &AffixRegistry,
) {
    let row_height = Val::Px(28.0);
    let font_size = 18.0;

    match (old, new) {
        (None, None) => {
            let child = commands
                .spawn((
                    Text::new("[empty]"),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.4, 0.4)),
                    Node {
                        height: row_height,
                        ..default()
                    },
                ))
                .id();
            commands.entity(parent_entity).add_child(child);
        }
        (None, Some(new_affix)) => {
            if let Some(def) = registry.get(new_affix.affix_id) {
                let child = commands
                    .spawn((
                        Text(def.format_display(new_affix.tier)),
                        TextFont {
                            font_size,
                            ..default()
                        },
                        TextColor(GREEN_COLOR),
                        Node {
                            height: row_height,
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(parent_entity).add_child(child);
            }
        }
        (Some(old_affix), None) => {
            if let Some(def) = registry.get(old_affix.affix_id) {
                let child = commands
                    .spawn((
                        Text(def.format_display(old_affix.tier)),
                        TextFont {
                            font_size,
                            ..default()
                        },
                        TextColor(RED_COLOR),
                        Node {
                            height: row_height,
                            ..default()
                        },
                    ))
                    .id();
                commands.entity(parent_entity).add_child(child);
            }
        }
        (Some(old_affix), Some(new_affix)) => {
            if old_affix.affix_id == new_affix.affix_id && old_affix.tier == new_affix.tier {
                if let Some(def) = registry.get(old_affix.affix_id) {
                    let child = commands
                        .spawn((
                            Text(def.format_display(old_affix.tier)),
                            TextFont {
                                font_size,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            Node {
                                height: row_height,
                                ..default()
                            },
                        ))
                        .id();
                    commands.entity(parent_entity).add_child(child);
                }
            } else if old_affix.affix_id == new_affix.affix_id
                && orb_behavior == OrbBehavior::Augmentation
            {
                if let Some(def) = registry.get(old_affix.affix_id) {
                    let old_display = def.format_value(old_affix.tier);
                    let new_display = def.format_value(new_affix.tier);
                    let text = format!(
                        "{}→{} [T{}→T{}]",
                        old_display,
                        new_display,
                        old_affix.tier + 1,
                        new_affix.tier + 1,
                    );
                    let child = commands
                        .spawn((
                            Text(text),
                            TextFont {
                                font_size,
                                ..default()
                            },
                            TextColor(GREEN_COLOR),
                            Node {
                                height: row_height,
                                ..default()
                            },
                        ))
                        .id();
                    commands.entity(parent_entity).add_child(child);
                }
            } else {
                let col = commands
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .id();
                if let Some(def) = registry.get(old_affix.affix_id) {
                    let old_child = commands
                        .spawn((
                            Text(def.format_display(old_affix.tier)),
                            TextFont {
                                font_size,
                                ..default()
                            },
                            TextColor(RED_COLOR),
                            Node {
                                height: row_height,
                                ..default()
                            },
                        ))
                        .id();
                    commands.entity(col).add_child(old_child);
                }
                if let Some(def) = registry.get(new_affix.affix_id) {
                    let new_child = commands
                        .spawn((
                            Text(def.format_display(new_affix.tier)),
                            TextFont {
                                font_size,
                                ..default()
                            },
                            TextColor(GREEN_COLOR),
                            Node {
                                height: row_height,
                                ..default()
                            },
                        ))
                        .id();
                    commands.entity(col).add_child(new_child);
                }
                commands.entity(parent_entity).add_child(col);
            }
        }
    }
}

fn get_spell_name(
    slot: SpellSlot,
    selected_spells: &SelectedSpells,
    blueprint_registry: &BlueprintRegistry,
) -> String {
    selected_spells
        .get(slot)
        .map(|id| blueprint_registry.get_display_name(id))
        .unwrap_or_else(|| "(none)".to_string())
}

pub fn buy_orb_system(
    interaction_query: Query<(&Interaction, &BuyOrbButton), Changed<Interaction>>,
    mut money: ResMut<PlayerMoney>,
    mut flow_state: ResMut<OrbFlowState>,
    orb_registry: Res<OrbRegistry>,
) {
    if !matches!(*flow_state, OrbFlowState::None) {
        return;
    }

    for (interaction, buy_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(def) = orb_registry.get(buy_btn.0) else {
            continue;
        };
        if money.0 >= def.price {
            money.0 -= def.price;
            *flow_state = OrbFlowState::SelectSlot { orb_id: buy_btn.0 };
        }
    }
}

pub fn slot_select_system(
    interaction_query: Query<(&Interaction, &SlotSelectButton), Changed<Interaction>>,
    mut flow_state: ResMut<OrbFlowState>,
    slot_query: Query<(&SpellSlotTag, &Affixes)>,
    orb_registry: Res<OrbRegistry>,
    affix_registry: Res<AffixRegistry>,
) {
    let OrbFlowState::SelectSlot { orb_id } = *flow_state else {
        return;
    };

    for (interaction, slot_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok((slot_tag, affixes)) = slot_query.get(slot_btn.0) else {
            continue;
        };
        let Some(orb_def) = orb_registry.get(orb_id) else {
            continue;
        };

        let original = affixes.clone();
        let mut preview = affixes.clone();
        let pool = affix_registry.pool(slot_tag.0);
        let mut rng = rand::rng();

        match orb_def.behavior {
            OrbBehavior::Alteration => apply_alteration(&mut preview, pool, &mut rng),
            OrbBehavior::Chaos => apply_chaos(&mut preview, pool, &mut rng),
            OrbBehavior::Augmentation => apply_augmentation(&mut preview, &affix_registry, &mut rng),
        }

        *flow_state = OrbFlowState::Preview {
            orb_id,
            slot_entity: slot_btn.0,
            slot_type: slot_tag.0,
            original,
            preview,
        };
        break;
    }
}

pub fn accept_orb_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<AcceptOrbButton>)>,
    mut flow_state: ResMut<OrbFlowState>,
    mut slot_query: Query<(&SlotOwner, &mut Affixes)>,
    affix_registry: Res<AffixRegistry>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats), With<Player>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let OrbFlowState::Preview {
            slot_entity,
            ref preview,
            ..
        } = *flow_state
        else {
            continue;
        };

        let preview_clone = preview.clone();

        if let Ok((owner, mut affixes)) = slot_query.get_mut(slot_entity) {
            *affixes = preview_clone;
            if let Ok((mut modifiers, mut dirty)) = player_query.get_mut(owner.0) {
                sync_affix_modifiers(
                    slot_entity,
                    &affixes,
                    &affix_registry,
                    &mut modifiers,
                    &mut dirty,
                );
            }
        }

        *flow_state = OrbFlowState::None;
        break;
    }
}

pub fn cancel_orb_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CancelOrbButton>)>,
    mut flow_state: ResMut<OrbFlowState>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        *flow_state = OrbFlowState::None;
        break;
    }
}

pub fn manage_orb_popup(
    mut commands: Commands,
    flow_state: Res<OrbFlowState>,
    popup_query: Query<Entity, With<OrbPopup>>,
    slot_query: Query<(Entity, &SpellSlotTag)>,
    selected_spells: Res<SelectedSpells>,
    blueprint_registry: Res<BlueprintRegistry>,
    affix_registry: Res<AffixRegistry>,
    orb_registry: Res<OrbRegistry>,
) {
    if !flow_state.is_changed() {
        return;
    }

    for entity in &popup_query {
        commands.entity(entity).despawn();
    }

    match &*flow_state {
        OrbFlowState::None => {}
        OrbFlowState::SelectSlot { .. } => {
            spawn_slot_select_popup(
                &mut commands,
                &slot_query,
                &selected_spells,
                &blueprint_registry,
            );
        }
        OrbFlowState::Preview {
            orb_id,
            slot_type,
            original,
            preview,
            ..
        } => {
            let behavior = orb_registry
                .get(*orb_id)
                .map(|d| d.behavior)
                .unwrap_or(OrbBehavior::Alteration);

            spawn_preview_popup(
                &mut commands,
                original,
                preview,
                *slot_type,
                behavior,
                &affix_registry,
                &selected_spells,
                &blueprint_registry,
            );
        }
    }
}

pub fn update_orb_button_colors(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Or<(
                With<BuyOrbButton>,
                With<SlotSelectButton>,
                With<AcceptOrbButton>,
                With<CancelOrbButton>,
            )>,
            Changed<Interaction>,
        ),
    >,
) {
    for (interaction, mut color) in &mut query {
        match interaction {
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
            _ => {}
        }
    }
}

pub fn update_orb_tooltip(
    mut commands: Commands,
    targets: Query<(Entity, &Interaction, &OrbTooltipTarget, &UiGlobalTransform, &ComputedNode)>,
    existing: Query<Entity, With<OrbTooltip>>,
    orb_registry: Res<OrbRegistry>,
    mut last_hovered: Local<Option<(OrbId, Entity)>>,
) {
    let mut hovered = None;
    let mut hovered_center = Vec2::ZERO;
    let mut hovered_size = Vec2::ZERO;
    let mut inv_scale = 1.0_f32;
    for (entity, interaction, target, gt, cn) in &targets {
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            hovered = Some((target.0, entity));
            hovered_center = gt.translation;
            hovered_size = cn.size();
            inv_scale = cn.inverse_scale_factor();
            break;
        }
    }

    if hovered == *last_hovered {
        return;
    }
    *last_hovered = hovered;

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Some((orb_id, _)) = hovered else {
        return;
    };
    let Some(def) = orb_registry.get(orb_id) else {
        return;
    };

    const TOOLTIP_WIDTH: f32 = 260.0;
    let top_left = (hovered_center - hovered_size / 2.0) * inv_scale;
    let left = (top_left.x - TOOLTIP_WIDTH - 8.0).max(0.0);
    let top = top_left.y.max(0.0);

    commands
        .spawn((
            OrbTooltip,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                max_width: Val::Px(TOOLTIP_WIDTH),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.12, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(&def.name),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(GOLD_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(&def.description),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}
