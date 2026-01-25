mod spawn_projectile;
mod damage;

pub use spawn_projectile::{SpawnProjectileEffect, Projectile};
pub use damage::DamageEffect;

use super::registry::EffectRegistry;

pub fn register_effects(registry: &mut EffectRegistry) {
    registry.register("spawn_projectile", SpawnProjectileEffect);
    registry.register("damage", DamageEffect);
}
