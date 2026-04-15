use bevy::prelude::*;
use rand::Rng;

use super::data::{RuneStub, ShopOffer};

pub fn fill_shop_offer(mut offer: ResMut<ShopOffer>) {
    let mut rng = rand::rng();
    for slot in offer.runes.iter_mut() {
        if slot.is_some() {
            continue;
        }
        let hue = rng.random_range(0.0..360.0);
        let color = Color::hsl(hue, 0.75, 0.55);
        let id = 0;
        *slot = Some(RuneStub { id, color });
    }
    let mut next_id = offer.next_id;
    for slot in offer.runes.iter_mut() {
        if let Some(stub) = slot {
            if stub.id == 0 {
                stub.id = next_id;
                next_id += 1;
            }
        }
    }
    offer.next_id = next_id;
}
