use bevy::prelude::*;
use rand::Rng;

use super::data::{RuneStub, ShopOffer, StubKind};

const JOKER_PROBABILITY: f32 = 0.25;
const JOKER_HUE_MIN: f32 = 35.0;
const JOKER_HUE_MAX: f32 = 60.0;

pub fn fill_shop_offer(mut offer: ResMut<ShopOffer>) {
    let mut rng = rand::rng();
    for slot in offer.stubs.iter_mut() {
        if slot.is_some() {
            continue;
        }
        let kind = if rng.random::<f32>() < JOKER_PROBABILITY {
            StubKind::Joker
        } else {
            StubKind::Rune
        };
        let color = match kind {
            StubKind::Rune => {
                let safe_range = 360.0 - (JOKER_HUE_MAX - JOKER_HUE_MIN);
                let raw = rng.random_range(0.0..safe_range);
                let hue = if raw < JOKER_HUE_MIN {
                    raw
                } else {
                    raw + (JOKER_HUE_MAX - JOKER_HUE_MIN)
                };
                Color::hsl(hue, 0.75, 0.55)
            }
            StubKind::Joker => {
                Color::hsl(rng.random_range(JOKER_HUE_MIN..JOKER_HUE_MAX), 0.85, 0.6)
            }
        };
        *slot = Some(RuneStub { id: 0, color, kind });
    }
    let mut next_id = offer.next_id;
    for slot in offer.stubs.iter_mut() {
        if let Some(stub) = slot {
            if stub.id == 0 {
                stub.id = next_id;
                next_id += 1;
            }
        }
    }
    offer.next_id = next_id;
}
