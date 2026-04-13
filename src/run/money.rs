use bevy::prelude::*;

pub fn register(app: &mut App) {
    app.init_resource::<PlayerMoney>();
}

#[derive(Resource, Default)]
pub struct PlayerMoney(u32);

impl PlayerMoney {
    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn earn(&mut self, amount: u32) {
        self.0 += amount;
    }

    #[expect(dead_code, reason = "shop not wired yet — only earn() is used")]
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
}
