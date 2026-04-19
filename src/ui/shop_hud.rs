use bevy::prelude::*;

use crate::balance::{Globals, RuneCosts};
use crate::palette;
use crate::run::{PlayerMoney, RunState};
use crate::rune::{
    roll_shop_offer, Dragging, RerollState, RuneGrid, ShopOffer,
};
use crate::transition::{Transition, TransitionAction};
use crate::wave::WavePhase;

use super::widgets::{button_node, panel_node, ReleasedButtons};

const PANEL_W: f32 = 220.0;
const PANEL_RIGHT: f32 = 40.0;
const PANEL_TOP: f32 = 24.0;
const PANEL_GAP: f32 = 12.0;
const BTN_H: f32 = 60.0;
const REROLL_H: f32 = 48.0;
const HEADER_FONT: f32 = 22.0;
const COIN_FONT: f32 = 24.0;
const BTN_FONT: f32 = 22.0;
const REROLL_FONT: f32 = 18.0;

#[derive(Component)]
pub struct ShopHudRoot;

#[derive(Component)]
pub struct ShopWaveText;

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct RerollButton;

#[derive(Component)]
pub struct RerollButtonLabel;

pub fn register(app: &mut App) {
    app.add_systems(OnEnter(WavePhase::Shop), spawn_shop_hud)
        .add_systems(
            Update,
            (
                start_run_system,
                reroll_button_system,
                update_reroll_label,
                update_coins_text,
                update_wave_text,
            )
                .run_if(in_state(WavePhase::Shop)),
        );
}

pub fn spawn_shop_hud(
    mut commands: Commands,
    run_state: Res<RunState>,
    money: Res<PlayerMoney>,
) {
    let root = commands
        .spawn((
            Name::new("ShopHudRoot"),
            ShopHudRoot,
            DespawnOnExit(WavePhase::Shop),
            GlobalZIndex(50),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(PANEL_RIGHT),
                top: Val::Px(PANEL_TOP),
                width: Val::Px(PANEL_W),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(PANEL_GAP),
                align_items: AlignItems::Stretch,
                ..default()
            },
        ))
        .id();

    let panel = commands
        .spawn((
            ChildOf(root),
            panel_node(
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(6.0),
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                None,
            ),
        ))
        .id();

    commands.spawn((
        ChildOf(panel),
        ShopWaveText,
        Text(format!("Wave {}", run_state.wave)),
        TextFont { font_size: HEADER_FONT, ..default() },
        TextColor(palette::color("ui_text")),
    ));
    commands.spawn((
        ChildOf(panel),
        ShopCoinsText,
        Text(format!("Coins: {}", money.get())),
        TextFont { font_size: COIN_FONT, ..default() },
        TextColor(palette::color("ui_text_money")),
    ));

    commands.spawn((
        ChildOf(root),
        StartRunButton,
        button_node(
            Node {
                width: Val::Px(PANEL_W),
                height: Val::Px(BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        ),
        children![(
            Text::new("Start Run"),
            TextFont { font_size: BTN_FONT, ..default() },
            TextColor(palette::color("ui_text")),
        )],
    ));

    commands.spawn((
        ChildOf(root),
        RerollButton,
        button_node(
            Node {
                width: Val::Px(PANEL_W),
                height: Val::Px(REROLL_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        ),
        children![(
            RerollButtonLabel,
            Text::new(""),
            TextFont { font_size: REROLL_FONT, ..default() },
            TextColor(palette::color("ui_text")),
        )],
    ));
}

fn start_run_system(
    buttons: ReleasedButtons<StartRunButton>,
    mut transition: ResMut<Transition>,
) {
    buttons.for_each(|_| {
        transition.request(TransitionAction::Wave(WavePhase::Combat));
    });
}

#[allow(clippy::too_many_arguments)]
fn reroll_button_system(
    buttons: ReleasedButtons<RerollButton>,
    mut money: ResMut<PlayerMoney>,
    mut offer: ResMut<ShopOffer>,
    grid: Res<RuneGrid>,
    globals: Res<Globals>,
    costs: Res<RuneCosts>,
    mut reroll: ResMut<RerollState>,
    dragging: Query<(), With<Dragging>>,
) {
    buttons.for_each(|_| {
        if !dragging.is_empty() {
            return;
        }
        if !money.can_afford(reroll.cost) {
            return;
        }
        money.spend(reroll.cost);
        roll_shop_offer(&mut offer, &grid, &globals, &costs);
        reroll.cost = reroll.cost.saturating_add(globals.rune_reroll_cost_step);
    });
}

fn update_reroll_label(
    reroll: Res<RerollState>,
    money: Res<PlayerMoney>,
    buttons: Query<&Children, With<RerollButton>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<RerollButtonLabel>>,
) {
    if !reroll.is_changed() && !money.is_changed() {
        return;
    }
    let affordable = money.can_afford(reroll.cost);
    for children in &buttons {
        for c in children.iter() {
            let Ok((mut text, mut color)) = texts.get_mut(c) else { continue };
            text.0 = format!("Reroll ({})", reroll.cost);
            *color = TextColor(if affordable {
                palette::color("ui_text")
            } else {
                palette::color("ui_text_disabled")
            });
        }
    }
}

fn update_coins_text(
    money: Res<PlayerMoney>,
    mut texts: Query<&mut Text, With<ShopCoinsText>>,
) {
    if !money.is_changed() {
        return;
    }
    for mut text in &mut texts {
        text.0 = format!("Coins: {}", money.get());
    }
}

fn update_wave_text(
    run_state: Res<RunState>,
    mut texts: Query<&mut Text, With<ShopWaveText>>,
) {
    if !run_state.is_changed() {
        return;
    }
    for mut text in &mut texts {
        text.0 = format!("Wave {}", run_state.wave);
    }
}
