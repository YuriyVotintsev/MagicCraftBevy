use bevy::prelude::*;
use magic_craft_macros::ability_component;

#[ability_component]
pub struct Health {
    pub max: ScalarExpr,
    #[raw(default = 0.0)]
    pub current: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_health);
}

fn init_health(mut query: Query<&mut Health, Added<Health>>) {
    for mut health in &mut query {
        if health.current <= 0.0 {
            health.current = health.max;
        }
    }
}
