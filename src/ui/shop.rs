use bevy::prelude::*;

use crate::affixes::{OrbFlowState, OrbRegistry};
use crate::artifacts::{
    Artifact, ArtifactId, ArtifactRegistry, BuyRequest, PlayerArtifacts, RerollCost,
    RerollRequest, ShopOfferings,
};
use crate::balance::GameBalance;
use crate::money::PlayerMoney;
use crate::ui::affix_shop::{self, OrbSection};
use crate::ui::artifact_tooltip::ArtifactTooltipTarget;
use crate::wave::{WavePhase, WaveState};

#[derive(Component)]
pub struct ShopRoot;

#[derive(Component)]
pub struct NextWaveButton;

#[derive(Component)]
pub struct BuyButton {
    pub index: usize,
}

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct ShopSection;

#[derive(Component)]
pub struct RerollButton;

#[derive(Component)]
pub struct RerollText;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const DISABLED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const SECTION_BG: Color = Color::srgba(0.15, 0.15, 0.25, 0.9);

fn section_header(label: &str) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(GOLD_COLOR),
        Node {
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
    )
}

fn shop_row(name: &str, price: u32, index: usize, can_buy: bool, artifact_id: ArtifactId) -> impl Bundle {
    (
        Interaction::default(),
        ArtifactTooltipTarget(artifact_id),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(6.0)),
            ..default()
        },
        children![
            (
                Text(format!("{} - {}g", name, price)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::right(Val::Px(20.0)),
                    ..default()
                },
            ),
            (
                Button,
                BuyButton { index },
                ArtifactTooltipTarget(artifact_id),
                Node {
                    width: Val::Px(70.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if can_buy { NORMAL_BUTTON } else { DISABLED_BUTTON }),
                children![(
                    Text::new("Buy"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(if can_buy { TEXT_COLOR } else { Color::srgb(0.4, 0.4, 0.4) })
                )]
            )
        ],
    )
}

fn build_shop_section(
    commands: &mut Commands,
    section: Entity,
    offerings: &[Entity],
    money: u32,
    artifacts: &PlayerArtifacts,
    registry: &ArtifactRegistry,
    artifact_query: &Query<&Artifact>,
) {
    let header = commands.spawn(section_header("SHOP")).id();
    commands.entity(section).add_child(header);

    if offerings.is_empty() {
        let empty = commands
            .spawn((
                Text::new("(sold out)"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ))
            .id();
        commands.entity(section).add_child(empty);
    } else {
        for (i, &artifact_entity) in offerings.iter().enumerate() {
            let Ok(artifact) = artifact_query.get(artifact_entity) else {
                continue;
            };
            if let Some(def) = registry.get(artifact.0) {
                let can_buy = money >= def.price && !artifacts.is_full();
                let row = commands.spawn(shop_row(&def.name, def.price, i, can_buy, artifact.0)).id();
                commands.entity(section).add_child(row);
            }
        }
    }
}

pub fn spawn_shop(
    mut commands: Commands,
    wave_state: Res<WaveState>,
    money: Res<PlayerMoney>,
    offerings: Res<ShopOfferings>,
    artifacts: Res<PlayerArtifacts>,
    registry: Res<ArtifactRegistry>,
    artifact_query: Query<&Artifact>,
    orb_registry: Res<OrbRegistry>,
    flow_state: Res<OrbFlowState>,
    mut reroll_cost: ResMut<RerollCost>,
    balance: Res<GameBalance>,
) {
    reroll_cost.reset_to(balance.shop.base_reroll_cost);

    let shop_section = commands
        .spawn((
            ShopSection,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(16.0)),
                width: Val::Px(400.0),
                ..default()
            },
            BackgroundColor(SECTION_BG),
        ))
        .id();
    build_shop_section(
        &mut commands,
        shop_section,
        offerings.as_slice(),
        money.get(),
        &artifacts,
        &registry,
        &artifact_query,
    );

    let orb_section = commands
        .spawn((
            OrbSection,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(16.0)),
                width: Val::Px(400.0),
                ..default()
            },
            BackgroundColor(SECTION_BG),
        ))
        .id();
    affix_shop::build_orb_section(
        &mut commands,
        orb_section,
        &orb_registry,
        money.get(),
        &flow_state,
    );

    let shops_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(16.0),
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        })
        .add_children(&[shop_section, orb_section])
        .id();

    let header = commands
        .spawn((
            Text(format!("Wave {} Complete!", wave_state.current_wave)),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ))
        .id();

    let money_text = commands
        .spawn((
            MoneyText,
            Text(format!("Total: {} coins", money.get())),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ))
        .id();

    let can_reroll = money.can_afford(reroll_cost.get());
    let reroll_btn = commands
        .spawn((
            Button,
            RerollButton,
            Node {
                width: Val::Px(140.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(if can_reroll { NORMAL_BUTTON } else { DISABLED_BUTTON }),
        ))
        .with_children(|parent| {
            parent.spawn((
                RerollText,
                Text(format!("Reroll - {}g", reroll_cost.get())),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(if can_reroll {
                    TEXT_COLOR
                } else {
                    Color::srgb(0.4, 0.4, 0.4)
                }),
            ));
        })
        .id();

    let next_wave_btn = commands
        .spawn((
            Button,
            NextWaveButton,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Next Wave"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        })
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.95)),
        ))
        .add_children(&[header, money_text, reroll_btn, shops_row, next_wave_btn])
        .id();

    commands
        .spawn((
            Name::new("ShopRoot"),
            ShopRoot,
            DespawnOnExit(WavePhase::Shop),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .add_children(&[panel]);
}

pub fn buy_system(
    interaction_query: Query<(&Interaction, &BuyButton), Changed<Interaction>>,
    mut buy_events: MessageWriter<BuyRequest>,
) {
    for (interaction, buy_btn) in &interaction_query {
        if *interaction == Interaction::Pressed {
            buy_events.write(BuyRequest { index: buy_btn.index });
        }
    }
}

pub fn reroll_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RerollButton>),
    >,
    mut money: ResMut<PlayerMoney>,
    mut reroll_cost: ResMut<RerollCost>,
    mut reroll_events: MessageWriter<RerollRequest>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                if money.spend(reroll_cost.get()) {
                    reroll_cost.increment();
                    reroll_events.write(RerollRequest);
                }
            }
            Interaction::Hovered => {
                if money.can_afford(reroll_cost.get()) {
                    *color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                *color = if money.can_afford(reroll_cost.get()) {
                    NORMAL_BUTTON
                } else {
                    DISABLED_BUTTON
                }
                .into();
            }
        }
    }
}

pub fn update_shop_on_change(
    mut commands: Commands,
    mut money_text: Query<&mut Text, (With<MoneyText>, Without<RerollText>)>,
    shop_section: Query<Entity, With<ShopSection>>,
    orb_section_query: Query<Entity, With<OrbSection>>,
    mut reroll_text: Query<(&mut Text, &mut TextColor), (With<RerollText>, Without<MoneyText>)>,
    mut reroll_btn: Query<&mut BackgroundColor, With<RerollButton>>,
    money: Res<PlayerMoney>,
    artifacts: Res<PlayerArtifacts>,
    offerings: Res<ShopOfferings>,
    registry: Res<ArtifactRegistry>,
    artifact_query: Query<&Artifact>,
    orb_registry: Res<OrbRegistry>,
    flow_state: Res<OrbFlowState>,
    reroll_cost: Res<RerollCost>,
) {
    if !money.is_changed()
        && !artifacts.is_changed()
        && !offerings.is_changed()
        && !flow_state.is_changed()
        && !reroll_cost.is_changed()
    {
        return;
    }

    for mut text in &mut money_text {
        text.0 = format!("Total: {} coins", money.get());
    }

    let can_reroll = money.can_afford(reroll_cost.get());
    for (mut text, mut color) in &mut reroll_text {
        text.0 = format!("Reroll - {}g", reroll_cost.get());
        *color = TextColor(if can_reroll {
            TEXT_COLOR
        } else {
            Color::srgb(0.4, 0.4, 0.4)
        });
    }
    for mut bg in &mut reroll_btn {
        *bg = BackgroundColor(if can_reroll {
            NORMAL_BUTTON
        } else {
            DISABLED_BUTTON
        });
    }

    for entity in &shop_section {
        commands.entity(entity).despawn_children();
        build_shop_section(
            &mut commands,
            entity,
            offerings.as_slice(),
            money.get(),
            &artifacts,
            &registry,
            &artifact_query,
        );
    }

    for entity in &orb_section_query {
        commands.entity(entity).despawn_children();
        affix_shop::build_orb_section(
            &mut commands,
            entity,
            &orb_registry,
            money.get(),
            &flow_state,
        );
    }
}

pub fn next_wave_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<NextWaveButton>),
    >,
    mut next_phase: ResMut<NextState<WavePhase>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_phase.set(WavePhase::Combat);
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}

pub fn update_button_colors(
    mut buy_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<BuyButton>, Without<NextWaveButton>),
    >,
) {
    for (interaction, mut color) in &mut buy_query {
        match interaction {
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
            _ => {}
        }
    }
}
