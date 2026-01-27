mod spawn_projectile;
mod damage;
mod orbiting;
mod dash;
mod meteor;

pub use spawn_projectile::{SpawnProjectileEffect, Projectile, Pierce};
pub use damage::DamageEffect;
pub use orbiting::{OrbitingMovement, SpawnOrbitingEffect};
pub use dash::{DashEffect, Dashing};
pub use meteor::{
    SpawnMeteorEffect, MeteorRequest, MeteorFalling, MeteorIndicator, MeteorExplosion,
};

use super::registry::EffectRegistry;

pub fn register_effects(registry: &mut EffectRegistry) {
    registry.register("spawn_projectile", SpawnProjectileEffect);
    registry.register("spawn_orbiting", SpawnOrbitingEffect);
    registry.register("damage", DamageEffect);
    registry.register("dash", DashEffect);
    registry.register("spawn_meteor", SpawnMeteorEffect);
}
