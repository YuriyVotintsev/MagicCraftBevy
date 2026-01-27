use bevy::prelude::*;

use crate::abilities::AbilityRegistry;
use crate::abilities::ids::AbilityId;
use crate::game_state::GameState;
use crate::player::{SelectedSpells, SpellSlot};

#[derive(Component)]
pub struct SpellButton {
    pub slot: SpellSlot,
    pub ability_id: AbilityId,
}

#[derive(Component)]
pub struct StartButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const SELECTED_BUTTON: Color = Color::srgb(0.2, 0.5, 0.2);
const SELECTED_HOVERED_BUTTON: Color = Color::srgb(0.25, 0.6, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub fn spawn_spell_selection(
    mut commands: Commands,
    mut selected_spells: ResMut<SelectedSpells>,
    ability_registry: Res<AbilityRegistry>,
) {
    selected_spells.randomize(&ability_registry);

    let root = commands.spawn((
        Name::new("SpellSelectionRoot"),
        DespawnOnExit(GameState::SpellSelection),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).id();

    let container = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
    )).id();

    commands.entity(root).add_child(container);

    let title = commands.spawn((
        Text::new("Choose Your Spells"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            margin: UiRect::bottom(Val::Px(30.0)),
            ..default()
        }
    )).id();

    commands.entity(container).add_child(title);

    let columns_container = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(30.0)),
            ..default()
        },
    )).id();

    commands.entity(container).add_child(columns_container);

    for slot in [SpellSlot::Active, SpellSlot::Passive, SpellSlot::Defensive] {
        let column = spawn_spell_column(&mut commands, slot, &ability_registry);
        commands.entity(columns_container).add_child(column);
    }

    let start_btn = commands.spawn((
        Button,
        StartButton,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            margin: UiRect::top(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
    )).id();

    commands.entity(container).add_child(start_btn);

    let start_text = commands.spawn((
        Text::new("Start Game"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(TEXT_COLOR)
    )).id();

    commands.entity(start_btn).add_child(start_text);
}

fn spawn_spell_column(
    commands: &mut Commands,
    slot: SpellSlot,
    ability_registry: &AbilityRegistry,
) -> Entity {
    let column = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
    )).id();

    let label = commands.spawn((
        Text::new(slot.label()),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            margin: UiRect::bottom(Val::Px(15.0)),
            ..default()
        }
    )).id();

    commands.entity(column).add_child(label);

    for &ability_name in slot.choices() {
        let button = spawn_spell_button(commands, slot, ability_name, ability_registry);
        commands.entity(column).add_child(button);
    }

    column
}

fn spawn_spell_button(
    commands: &mut Commands,
    slot: SpellSlot,
    ability_name: &str,
    ability_registry: &AbilityRegistry,
) -> Entity {
    let ability_id = ability_registry.get_id(ability_name).unwrap_or_default();
    let display_name = format_ability_name(ability_name);

    let button = commands.spawn((
        Button,
        SpellButton {
            slot,
            ability_id,
        },
        Node {
            width: Val::Px(180.0),
            height: Val::Px(50.0),
            margin: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
    )).id();

    let text = commands.spawn((
        Text::new(display_name),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR)
    )).id();

    commands.entity(button).add_child(text);

    button
}

fn format_ability_name(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn spell_button_system(
    mut interaction_query: Query<
        (&Interaction, &SpellButton),
        Changed<Interaction>,
    >,
    mut selected_spells: ResMut<SelectedSpells>,
) {
    for (interaction, spell_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            selected_spells.set(spell_button.slot, spell_button.ability_id);
        }
    }
}

pub fn update_spell_button_colors(
    mut button_query: Query<(&Interaction, &SpellButton, &mut BackgroundColor)>,
    selected_spells: Res<SelectedSpells>,
) {
    for (interaction, spell_button, mut color) in &mut button_query {
        let is_selected = selected_spells.get(spell_button.slot) == Some(spell_button.ability_id);

        *color = match (*interaction, is_selected) {
            (Interaction::Pressed, _) => SELECTED_BUTTON.into(),
            (Interaction::Hovered, true) => SELECTED_HOVERED_BUTTON.into(),
            (Interaction::Hovered, false) => HOVERED_BUTTON.into(),
            (Interaction::None, true) => SELECTED_BUTTON.into(),
            (Interaction::None, false) => NORMAL_BUTTON.into(),
        };
    }
}

pub fn start_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartButton>),
    >,
    selected_spells: Res<SelectedSpells>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                if selected_spells.is_complete() {
                    next_state.set(GameState::Playing);
                }
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
