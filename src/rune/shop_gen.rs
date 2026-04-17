use bevy::prelude::*;
use rand::Rng;

use crate::balance::GameBalance;

use super::content::{RuneKind, Tier};
use super::cost::RuneCosts;
use super::data::{Rune, RuneGrid, ShopOffer};

const JOKER_HUE_MIN: f32 = 35.0;
const JOKER_HUE_MAX: f32 = 60.0;

const TIER_ROLL_ATTEMPTS: u32 = 8;

fn joker_cost_for_tier(tier: Tier) -> u32 {
    match tier {
        Tier::Common => 4,
        Tier::Rare => 7,
    }
}

fn random_tier(rng: &mut impl Rng, balance: &GameBalance) -> Option<Tier> {
    let total = balance.runes.tier_weights.total();
    if total == 0 {
        return None;
    }
    let mut pick = rng.random_range(0..total);
    for tier in Tier::ALL.iter().copied() {
        let w = balance.runes.tier_weights.for_tier(tier);
        if pick < w {
            return Some(tier);
        }
        pick -= w;
    }
    None
}

fn count_kind_in_grid(grid: &RuneGrid, kind: RuneKind) -> u32 {
    grid.cells
        .values()
        .filter(|r| r.kind == Some(kind))
        .count() as u32
}

fn count_kind_in_offer(offer: &ShopOffer, kind: RuneKind) -> u32 {
    offer
        .stubs
        .iter()
        .filter_map(|s| s.as_ref())
        .filter(|r| r.kind == Some(kind))
        .count() as u32
}

fn pick_rune_for_tier(
    rng: &mut impl Rng,
    tier: Tier,
    grid: &RuneGrid,
    offer: &ShopOffer,
) -> Option<RuneKind> {
    let candidates: Vec<RuneKind> = RuneKind::ALL
        .iter()
        .copied()
        .filter(|k| k.def().tier == tier)
        .filter(|k| {
            let limit = k.def().limit.unwrap_or(u32::MAX);
            count_kind_in_grid(grid, *k) + count_kind_in_offer(offer, *k) < limit
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let idx = rng.random_range(0..candidates.len());
    Some(candidates[idx])
}

fn roll_rune(
    rng: &mut impl Rng,
    balance: &GameBalance,
    costs: &RuneCosts,
    grid: &RuneGrid,
    offer: &ShopOffer,
) -> Option<Rune> {
    for _ in 0..TIER_ROLL_ATTEMPTS {
        let Some(tier) = random_tier(rng, balance) else {
            return None;
        };
        if let Some(kind) = pick_rune_for_tier(rng, tier, grid, offer) {
            let def = kind.def();
            let (r, g, b) = crate::palette::lookup(def.palette_key).unwrap_or((0.5, 0.5, 0.5));
            return Some(Rune {
                id: 0,
                color: Color::srgb(r, g, b),
                tier: def.tier,
                kind: Some(kind),
                cost: costs.cost_for(kind),
            });
        }
    }
    None
}

fn roll_joker(rng: &mut impl Rng, balance: &GameBalance) -> Option<Rune> {
    let tier = random_tier(rng, balance)?;
    let hue = rng.random_range(JOKER_HUE_MIN..JOKER_HUE_MAX);
    Some(Rune {
        id: 0,
        color: Color::hsl(hue, 0.85, 0.6),
        tier,
        kind: None,
        cost: joker_cost_for_tier(tier),
    })
}

pub fn roll_shop_offer(
    offer: &mut ShopOffer,
    grid: &RuneGrid,
    balance: &GameBalance,
    costs: &RuneCosts,
) {
    let mut rng = rand::rng();
    offer.stubs = [None; super::data::SHOP_SLOTS];
    for i in 0..offer.stubs.len() {
        let as_joker = rng.random::<f32>() < balance.runes.joker_probability;
        let new_rune = if as_joker {
            roll_joker(&mut rng, balance)
        } else {
            roll_rune(&mut rng, balance, costs, grid, offer)
        };
        offer.stubs[i] = new_rune;
    }

    let mut next_id = offer.next_id;
    for slot in offer.stubs.iter_mut() {
        if let Some(rune) = slot {
            if rune.id == 0 {
                rune.id = next_id;
                next_id += 1;
            }
        }
    }
    offer.next_id = next_id;
}

pub fn fill_shop_offer(
    mut offer: ResMut<ShopOffer>,
    grid: Res<RuneGrid>,
    balance: Res<GameBalance>,
    costs: Res<RuneCosts>,
) {
    roll_shop_offer(&mut offer, &grid, &balance, &costs);
}
