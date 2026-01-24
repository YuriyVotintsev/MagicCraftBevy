use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum StatId {
    Strength,
    StrengthIncreased,

    MaxLife,
    MaxLifeIncreased,
    MaxLifeMore,
    MaxLifePerStrength,

    MaxMana,
    MaxManaIncreased,

    PhysicalDamage,
    PhysicalDamageIncreased,
    PhysicalDamageMore,

    MovementSpeed,
    MovementSpeedIncreased,
    MovementSpeedMore,

    ProjectileSpeed,
    ProjectileSpeedIncreased,
    ProjectileCount,

    CritChance,
    CritChanceIncreased,
    CritMultiplier,
    CritMultiplierIncreased,
}
