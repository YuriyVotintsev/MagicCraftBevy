use bevy::prelude::*;

use crate::blueprints::{BlueprintId, BlueprintRegistry};
use crate::game_state::GameState;
use crate::player::{AvailableHeroes, SelectedHero};

#[derive(Component)]
pub struct HeroButton {
    pub hero_id: BlueprintId,
}

#[derive(Component)]
pub struct ContinueButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const SELECTED_BUTTON: Color = Color::srgb(0.2, 0.5, 0.2);
const SELECTED_HOVERED_BUTTON: Color = Color::srgb(0.25, 0.6, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub fn spawn_hero_selection(
    mut commands: Commands,
    available_heroes: Res<AvailableHeroes>,
    blueprint_registry: Res<BlueprintRegistry>,
    mut selected_hero: ResMut<SelectedHero>,
) {
    if let Some(&first_id) = available_heroes.0.first() {
        selected_hero.0 = Some(first_id);
    }

    let root = commands.spawn((
        Name::new("HeroSelectionRoot"),
        DespawnOnExit(GameState::HeroSelection),
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
        Text::new("Choose Your Hero"),
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

    let row = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(30.0)),
            ..default()
        },
    )).id();

    commands.entity(container).add_child(row);

    for &hero_id in &available_heroes.0 {
        let display_name = blueprint_registry.get_display_name(hero_id);

        let button = commands.spawn((
            Button,
            HeroButton { hero_id },
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
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
                font_size: 28.0,
                ..default()
            },
            TextColor(TEXT_COLOR)
        )).id();

        commands.entity(button).add_child(text);
        commands.entity(row).add_child(button);
    }

    let continue_btn = commands.spawn((
        Button,
        ContinueButton,
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

    commands.entity(container).add_child(continue_btn);

    let continue_text = commands.spawn((
        Text::new("Continue"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(TEXT_COLOR)
    )).id();

    commands.entity(continue_btn).add_child(continue_text);
}

pub fn hero_button_system(
    mut interaction_query: Query<
        (&Interaction, &HeroButton),
        Changed<Interaction>,
    >,
    mut selected_hero: ResMut<SelectedHero>,
) {
    for (interaction, hero_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            selected_hero.0 = Some(hero_button.hero_id);
        }
    }
}

pub fn update_hero_button_colors(
    mut button_query: Query<(&Interaction, &HeroButton, &mut BackgroundColor)>,
    selected_hero: Res<SelectedHero>,
) {
    for (interaction, hero_button, mut color) in &mut button_query {
        let is_selected = selected_hero.0 == Some(hero_button.hero_id);

        *color = match (*interaction, is_selected) {
            (Interaction::Pressed, _) => SELECTED_BUTTON.into(),
            (Interaction::Hovered, true) => SELECTED_HOVERED_BUTTON.into(),
            (Interaction::Hovered, false) => HOVERED_BUTTON.into(),
            (Interaction::None, true) => SELECTED_BUTTON.into(),
            (Interaction::None, false) => NORMAL_BUTTON.into(),
        };
    }
}

pub fn continue_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
    selected_hero: Res<SelectedHero>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                if selected_hero.0.is_some() {
                    next_state.set(GameState::SpellSelection);
                }
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
