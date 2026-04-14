pub mod visual {
    pub mod sprite;
    pub mod shadow;
    pub mod bobbing_animation;
    pub mod jump_walk_animation;
    pub mod shoot_squish;
    pub mod fade_out;
    pub mod growing;
    pub mod particles;
}

pub mod physics {
    pub mod collider;
    pub mod dynamic_body;
    pub mod static_body;
    pub mod size;
}

pub mod combat {
    pub mod projectile;
    pub mod damage_payload;
    pub mod melee_strike;
    pub mod melee_attacker;
    pub mod find_nearest_enemy;
    pub mod shot_fired;
}

pub mod player {
    pub mod keyboard_movement;
    pub mod player_input;
}

pub mod lifetime;

use bevy::prelude::*;

pub fn register_component_systems(app: &mut App) {
    visual::sprite::register_systems(app);
    visual::shadow::register_systems(app);
    visual::bobbing_animation::register_systems(app);
    visual::jump_walk_animation::register_systems(app);
    visual::shoot_squish::register_systems(app);
    visual::fade_out::register_systems(app);
    visual::growing::register_systems(app);
    visual::particles::register_systems(app);

    physics::collider::register_systems(app);
    physics::dynamic_body::register_systems(app);
    physics::static_body::register_systems(app);
    physics::size::register_systems(app);

    combat::projectile::register_systems(app);
    combat::damage_payload::register_systems(app);
    combat::melee_strike::register_systems(app);
    combat::melee_attacker::register_systems(app);
    combat::find_nearest_enemy::register_systems(app);
    combat::shot_fired::register_systems(app);

    player::keyboard_movement::register_systems(app);
    player::player_input::register_systems(app);

    lifetime::register_systems(app);
}
