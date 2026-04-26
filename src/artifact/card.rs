use bevy::prelude::*;

use super::kind::ArtifactKind;
use super::reroll::{RerollButton, RerollState};
use crate::palette;
use crate::run::BreatherTimer;
use crate::ui::widgets::{button_node, panel_node};

const CARD_WIDTH: f32 = 480.0;
const CARD_HEIGHT: f32 = 220.0;
const CARD_BORDER_PX: f32 = 4.0;

#[derive(Component)]
pub struct ArtifactCardRoot;

#[derive(Component)]
pub struct ArtifactCardData {
    pub kind: Option<ArtifactKind>,
    pub refresh: bool,
}

#[derive(Component)]
struct ArtifactCardBorder;

#[derive(Component)]
struct ArtifactCardName;

#[derive(Component)]
struct ArtifactCardDesc;

#[derive(Component)]
struct ArtifactCardIcon;

#[derive(Component)]
struct ArtifactCardRerollHost;

pub fn register(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_card_when_breather_starts,
            despawn_card_when_breather_ends,
            refresh_card_visuals,
            hide_reroll_when_used,
        ),
    );
}

fn spawn_card_when_breather_starts(
    mut commands: Commands,
    breather: Option<Res<BreatherTimer>>,
    mut last_active: Local<bool>,
    reroll: Res<RerollState>,
) {
    let active = breather.is_some();
    if active && !*last_active {
        spawn_card(&mut commands, &reroll);
    }
    *last_active = active;
}

fn despawn_card_when_breather_ends(
    mut commands: Commands,
    breather: Option<Res<BreatherTimer>>,
    mut last_active: Local<bool>,
    cards: Query<Entity, With<ArtifactCardRoot>>,
) {
    let active = breather.is_some();
    if !active && *last_active {
        for e in &cards {
            if let Ok(mut ec) = commands.get_entity(e) {
                ec.despawn();
            }
        }
    }
    *last_active = active;
}

fn spawn_card(commands: &mut Commands, reroll: &RerollState) {
    let kind = reroll.current;
    let tier_color = kind
        .map(|k| palette::color(k.def().tier.palette_key()))
        .unwrap_or_else(|| palette::color("ui_text_disabled"));

    let root = commands
        .spawn((
            Name::new("ArtifactCardRoot"),
            ArtifactCardRoot,
            ArtifactCardData {
                kind,
                refresh: true,
            },
            GlobalZIndex(60),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                padding: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    let card = commands
        .spawn((
            ChildOf(root),
            ArtifactCardBorder,
            panel_node(
                Node {
                    width: Val::Px(CARD_WIDTH),
                    height: Val::Px(CARD_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(CARD_BORDER_PX)),
                    ..default()
                },
                Some(tier_color),
            ),
        ))
        .id();

    commands.spawn((
        ChildOf(card),
        ArtifactCardIcon,
        Node {
            width: Val::Px(72.0),
            height: Val::Px(72.0),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(36.0)),
            ..default()
        },
        BackgroundColor(tier_color),
        BorderColor::all(palette::color("ui_panel_border")),
    ));

    commands.spawn((
        ChildOf(card),
        ArtifactCardName,
        Text::new(""),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(palette::color("ui_text_title")),
    ));

    commands.spawn((
        ChildOf(card),
        ArtifactCardDesc,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(palette::color("ui_text_subtle")),
    ));

    let reroll_host = commands
        .spawn((
            ChildOf(card),
            ArtifactCardRerollHost,
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(reroll_host),
        RerollButton,
        button_node(
            Node {
                width: Val::Px(160.0),
                height: Val::Px(44.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        ),
        children![(
            Text::new("Reroll"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(palette::color("ui_text")),
        )],
    ));
}

fn refresh_card_visuals(
    mut data_q: Query<(Entity, &mut ArtifactCardData)>,
    children_q: Query<&Children>,
    mut name_q: Query<&mut Text, (With<ArtifactCardName>, Without<ArtifactCardDesc>)>,
    mut desc_q: Query<&mut Text, (With<ArtifactCardDesc>, Without<ArtifactCardName>)>,
    mut border_q: Query<&mut BorderColor, With<ArtifactCardBorder>>,
    mut bg_q: Query<&mut BackgroundColor, With<ArtifactCardIcon>>,
) {
    for (root, mut data) in &mut data_q {
        if !data.refresh {
            continue;
        }
        data.refresh = false;
        let (name_str, desc_str, color) = match data.kind {
            Some(k) => {
                let d = k.def();
                (
                    d.name.to_string(),
                    d.description.to_string(),
                    palette::color(d.tier.palette_key()),
                )
            }
            None => (
                "Pool exhausted".to_string(),
                "No more artifacts to draw.".to_string(),
                palette::color("ui_text_disabled"),
            ),
        };

        let descendants = collect_descendants(root, &children_q);
        for d in &descendants {
            if let Ok(mut t) = name_q.get_mut(*d) {
                **t = name_str.clone();
            }
            if let Ok(mut t) = desc_q.get_mut(*d) {
                **t = desc_str.clone();
            }
            if let Ok(mut bg) = bg_q.get_mut(*d) {
                bg.0 = color;
            }
            if let Ok(mut bc) = border_q.get_mut(*d) {
                *bc = BorderColor::all(color);
            }
        }
    }
}

fn hide_reroll_when_used(
    reroll: Res<RerollState>,
    mut hosts: Query<&mut Node, With<ArtifactCardRerollHost>>,
) {
    if !reroll.is_changed() {
        return;
    }
    for mut node in &mut hosts {
        node.display = if reroll.available {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn collect_descendants(root: Entity, children_q: &Query<&Children>) -> Vec<Entity> {
    let mut out = vec![root];
    let mut idx = 0;
    while idx < out.len() {
        let e = out[idx];
        if let Ok(ch) = children_q.get(e) {
            for c in ch.iter() {
                out.push(c);
            }
        }
        idx += 1;
    }
    out
}
