use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;
use rand::Rng;

use crate::schedule::GameSet;
use crate::GameState;

#[ability_component]
pub struct Straight {
    pub speed: ScalarExpr,
    #[raw(default = 0)]
    pub spread: ScalarExpr,
    #[default_expr("target.direction")]
    pub direction: VecExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        init_straight
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_straight(
    mut commands: Commands,
    query: Query<(Entity, &Straight), Added<Straight>>,
) {
    for (entity, straight) in &query {
        let base_direction = straight.direction.normalize_or_zero();

        let direction = if straight.spread > 0.0 {
            let spread_rad = straight.spread.to_radians();
            let angle_offset = rand::rng().random_range(-spread_rad..spread_rad);
            let cos = angle_offset.cos();
            let sin = angle_offset.sin();
            Vec2::new(
                base_direction.x * cos - base_direction.y * sin,
                base_direction.x * sin + base_direction.y * cos,
            )
        } else {
            base_direction
        };

        commands.entity(entity).insert((
            RigidBody::Kinematic,
            LinearVelocity(direction * straight.speed),
        ));
    }
}
