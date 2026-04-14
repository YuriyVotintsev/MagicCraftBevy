use bevy::prelude::*;

mod attached_to;
mod caster;
mod damage;
mod damage_payload;
mod death;
mod find_nearest_enemy;
mod health;
mod melee_attacker;
mod melee_strike;
mod on_collision_damage;
mod projectile;
mod shot_fired;
mod target;

pub use caster::Caster;
pub use damage::PendingDamage;
pub use death::{death_system, Dead, DeathEvent, SkipCleanup};
pub use find_nearest_enemy::FindNearestEnemy;
pub use health::Health;
pub use melee_attacker::MeleeAttacker;
pub use on_collision_damage::OnCollisionDamage;
pub use projectile::{projectile_collision_physics, Projectile};
pub use shot_fired::ShotFired;
pub use target::Target;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        projectile::register_systems(app);
        damage_payload::register_systems(app);
        melee_strike::register_systems(app);
        melee_attacker::register_systems(app);
        find_nearest_enemy::register_systems(app);
        shot_fired::register_systems(app);
        on_collision_damage::register_systems(app);
        damage::register_systems(app);
        death::register_systems(app);
        attached_to::register_systems(app);
    }
}
