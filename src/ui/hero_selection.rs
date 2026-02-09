use bevy::prelude::*;

use crate::game_state::GameState;
use crate::player::{AvailableHeroes, SelectedHero};

#[derive(Component)]
pub struct HeroButton {
    pub index: usize,
}

#[derive(Component)]
pub struct ContinueButton;

#[derive(Component)]
pub struct StatsPanel;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const SELECTED_BUTTON: Color = Color::srgb(0.2, 0.5, 0.2);
const SELECTED_HOVERED_BUTTON: Color = Color::srgb(0.25, 0.6, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const POSITIVE_COLOR: Color = Color::srgb(0.3, 0.9, 0.3);
const NEGATIVE_COLOR: Color = Color::srgb(0.9, 0.3, 0.3);

fn format_stat_name(raw: &str) -> String {
    let name = raw
        .strip_suffix("_base").or_else(|| raw.strip_suffix("_more")).unwrap_or(raw);
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

fn format_modifier_value(value: f32) -> String {
    if value > 0.0 {
        if value.fract() == 0.0 {
            format!("+{}", value as i32)
        } else {
            format!("+{:.2}", value)
        }
    } else if value.fract() == 0.0 {
        format!("{}", value as i32)
    } else {
        format!("{:.2}", value)
    }
}

pub fn spawn_hero_selection(
    mut commands: Commands,
    available_heroes: Res<AvailableHeroes>,
    mut selected_hero: ResMut<SelectedHero>,
) {
    if !available_heroes.classes.is_empty() {
        selected_hero.0 = Some(0);
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
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        },
    )).id();

    commands.entity(container).add_child(row);

    for (i, class) in available_heroes.classes.iter().enumerate() {
        let button = commands.spawn((
            Button,
            HeroButton { index: i },
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
            Text::new(&class.display_name),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgba(class.color.0, class.color.1, class.color.2, class.color.3))
        )).id();

        commands.entity(button).add_child(text);
        commands.entity(row).add_child(button);
    }

    let stats_panel = commands.spawn((
        StatsPanel,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            padding: UiRect::all(Val::Px(15.0)),
            margin: UiRect::bottom(Val::Px(20.0)),
            min_width: Val::Px(280.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.15, 0.9)),
    )).id();

    commands.entity(container).add_child(stats_panel);

    let continue_btn = commands.spawn((
        Button,
        ContinueButton,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            margin: UiRect::top(Val::Px(0.0)),
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

pub fn update_stats_panel(
    mut commands: Commands,
    selected_hero: Res<SelectedHero>,
    available_heroes: Res<AvailableHeroes>,
    panel_query: Query<(Entity, Option<&Children>), With<StatsPanel>>,
) {
    if !selected_hero.is_changed() {
        return;
    }

    let Ok((panel_entity, children)) = panel_query.single() else {
        return;
    };

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let Some(index) = selected_hero.0 else { return };
    let Some(class) = available_heroes.classes.get(index) else { return };

    let header = commands.spawn((
        Text::new("Stat Modifiers"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        Node {
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
    )).id();
    commands.entity(panel_entity).add_child(header);

    let mut sorted_mods: Vec<_> = class.modifiers.iter().collect();
    sorted_mods.sort_by(|a, b| a.name.cmp(&b.name));

    for modifier in sorted_mods {
        let display_name = format_stat_name(&modifier.name);
        let value_str = format_modifier_value(modifier.value);
        let color = if modifier.value > 0.0 { POSITIVE_COLOR } else { NEGATIVE_COLOR };

        let row = commands.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
        )).id();

        let name_text = commands.spawn((
            Text::new(display_name),
            TextFont { font_size: 18.0, ..default() },
            TextColor(TEXT_COLOR),
        )).id();

        let value_text = commands.spawn((
            Text::new(value_str),
            TextFont { font_size: 18.0, ..default() },
            TextColor(color),
            Node {
                margin: UiRect::left(Val::Px(20.0)),
                ..default()
            },
        )).id();

        commands.entity(row).add_child(name_text);
        commands.entity(row).add_child(value_text);
        commands.entity(panel_entity).add_child(row);
    }
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
            selected_hero.0 = Some(hero_button.index);
        }
    }
}

pub fn update_hero_button_colors(
    mut button_query: Query<(&Interaction, &HeroButton, &mut BackgroundColor)>,
    selected_hero: Res<SelectedHero>,
) {
    for (interaction, hero_button, mut color) in &mut button_query {
        let is_selected = selected_hero.0 == Some(hero_button.index);

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
