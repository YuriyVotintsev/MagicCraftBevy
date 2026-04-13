pub mod common {
    pub mod size;
    pub mod collider;
    pub mod sprite;
    pub mod health;
    pub mod dynamic_body;
    pub mod static_body;
    pub mod jump_walk_animation;
    pub mod squish_walk_animation;
    pub mod shoot_squish;
    pub mod shadow;
    pub mod bobbing_animation;
    pub mod spinner_visual;
    pub mod tower_visual;
    pub mod fade_out;
    pub mod movement;
}

pub mod ability {
    pub mod projectile;
    pub mod lifetime;
    pub mod growing;
    pub mod damage_payload;
    pub mod find_nearest_enemy;
    pub mod melee_strike;
    pub mod particles;
}

pub mod mob {
    pub mod move_toward;
    pub mod lunge_movement;
    pub mod random_jump;
    pub mod ghost_transparency;
    pub mod spinner_windup;
    pub mod spinner_charge;
    pub mod jumper_ai;
    pub mod spinner_ai;
}

pub mod player {
    pub mod keyboard_movement;
    pub mod player_input;
}

use bevy::prelude::*;

pub fn register_component_systems(app: &mut App) {
    common::size::register_systems(app);
    common::collider::register_systems(app);
    common::sprite::register_systems(app);
    common::health::register_systems(app);
    common::dynamic_body::register_systems(app);
    common::static_body::register_systems(app);
    common::jump_walk_animation::register_systems(app);
    common::squish_walk_animation::register_systems(app);
    common::shoot_squish::register_systems(app);
    common::shadow::register_systems(app);
    common::bobbing_animation::register_systems(app);
    common::spinner_visual::register_systems(app);
    common::tower_visual::register_systems(app);
    common::fade_out::register_systems(app);

    ability::projectile::register_systems(app);
    ability::lifetime::register_systems(app);
    ability::growing::register_systems(app);
    ability::damage_payload::register_systems(app);
    ability::find_nearest_enemy::register_systems(app);
    ability::melee_strike::register_systems(app);
    ability::particles::register_systems(app);

    mob::move_toward::register_systems(app);
    mob::lunge_movement::register_systems(app);
    mob::random_jump::register_systems(app);
    mob::ghost_transparency::register_systems(app);
    mob::spinner_windup::register_systems(app);
    mob::spinner_charge::register_systems(app);
    mob::jumper_ai::register_systems(app);
    mob::spinner_ai::register_systems(app);

    player::keyboard_movement::register_systems(app);
    player::player_input::register_systems(app);
}
