mod spawn_projectile;
mod damage;
mod orbiting;
mod dash;

pub use spawn_projectile::{SpawnProjectileEffect, Projectile, Pierce};
pub use damage::DamageEffect;
pub use orbiting::{OrbitingMovement, SpawnOrbitingEffect};
pub use dash::{DashEffect, Dashing};

use super::registry::EffectRegistry;

pub fn register_effects(registry: &mut EffectRegistry) {
    registry.register("spawn_projectile", SpawnProjectileEffect);
    registry.register("spawn_orbiting", SpawnOrbitingEffect);
    registry.register("damage", DamageEffect);
    registry.register("dash", DashEffect);
}
