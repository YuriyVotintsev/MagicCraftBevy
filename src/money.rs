use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerMoney(u32);

impl PlayerMoney {
    pub fn new(amount: u32) -> Self {
        Self(amount)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn earn(&mut self, amount: u32) {
        self.0 += amount;
    }

    pub fn spend(&mut self, amount: u32) -> bool {
        if self.can_afford(amount) {
            self.0 -= amount;
            true
        } else {
            false
        }
    }

    pub fn can_afford(&self, price: u32) -> bool {
        self.0 >= price
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }
}
