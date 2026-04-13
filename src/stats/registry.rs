use bevy::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Reflect)]
pub enum Stat {
    MaxLifeFlat,
    MaxLifeIncreased,
    MaxLifeMore,
    MaxLife,
    MaxManaFlat,
    MaxManaIncreased,
    MaxMana,
    PhysicalDamageFlat,
    PhysicalDamageIncreased,
    PhysicalDamageMore,
    MovementSpeedFlat,
    MovementSpeedIncreased,
    MovementSpeed,
    ProjectileSpeedFlat,
    ProjectileSpeedIncreased,
    ProjectileCount,
    CritChanceFlat,
    CritChanceIncreased,
    CritChance,
    CritMultiplier,
    AreaOfEffectFlat,
    AreaOfEffectIncreased,
    DurationFlat,
    DurationIncreased,
    PickupRadiusFlat,
    PickupRadiusIncreased,
    PickupRadius,
}

impl Stat {
    pub const ALL: &'static [Stat] = &[
        Stat::MaxLifeFlat,
        Stat::MaxLifeIncreased,
        Stat::MaxLifeMore,
        Stat::MaxLife,
        Stat::MaxManaFlat,
        Stat::MaxManaIncreased,
        Stat::MaxMana,
        Stat::PhysicalDamageFlat,
        Stat::PhysicalDamageIncreased,
        Stat::PhysicalDamageMore,
        Stat::MovementSpeedFlat,
        Stat::MovementSpeedIncreased,
        Stat::MovementSpeed,
        Stat::ProjectileSpeedFlat,
        Stat::ProjectileSpeedIncreased,
        Stat::ProjectileCount,
        Stat::CritChanceFlat,
        Stat::CritChanceIncreased,
        Stat::CritChance,
        Stat::CritMultiplier,
        Stat::AreaOfEffectFlat,
        Stat::AreaOfEffectIncreased,
        Stat::DurationFlat,
        Stat::DurationIncreased,
        Stat::PickupRadiusFlat,
        Stat::PickupRadiusIncreased,
        Stat::PickupRadius,
    ];

    pub const COUNT: usize = Self::ALL.len();

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Stat::MaxLifeFlat => "max_life_flat",
            Stat::MaxLifeIncreased => "max_life_increased",
            Stat::MaxLifeMore => "max_life_more",
            Stat::MaxLife => "max_life",
            Stat::MaxManaFlat => "max_mana_flat",
            Stat::MaxManaIncreased => "max_mana_increased",
            Stat::MaxMana => "max_mana",
            Stat::PhysicalDamageFlat => "physical_damage_flat",
            Stat::PhysicalDamageIncreased => "physical_damage_increased",
            Stat::PhysicalDamageMore => "physical_damage_more",
            Stat::MovementSpeedFlat => "movement_speed_flat",
            Stat::MovementSpeedIncreased => "movement_speed_increased",
            Stat::MovementSpeed => "movement_speed",
            Stat::ProjectileSpeedFlat => "projectile_speed_flat",
            Stat::ProjectileSpeedIncreased => "projectile_speed_increased",
            Stat::ProjectileCount => "projectile_count",
            Stat::CritChanceFlat => "crit_chance_flat",
            Stat::CritChanceIncreased => "crit_chance_increased",
            Stat::CritChance => "crit_chance",
            Stat::CritMultiplier => "crit_multiplier",
            Stat::AreaOfEffectFlat => "area_of_effect_flat",
            Stat::AreaOfEffectIncreased => "area_of_effect_increased",
            Stat::DurationFlat => "duration_flat",
            Stat::DurationIncreased => "duration_increased",
            Stat::PickupRadiusFlat => "pickup_radius_flat",
            Stat::PickupRadiusIncreased => "pickup_radius_increased",
            Stat::PickupRadius => "pickup_radius",
        }
    }

    pub fn eval_kind(self) -> StatEvalKind {
        match self {
            Stat::MaxLifeMore | Stat::PhysicalDamageMore => StatEvalKind::Product,

            Stat::MaxLife => StatEvalKind::FlatIncreasedMore {
                flat: Stat::MaxLifeFlat,
                increased: Stat::MaxLifeIncreased,
                more: Stat::MaxLifeMore,
            },
            Stat::MaxMana => StatEvalKind::FlatIncreased {
                flat: Stat::MaxManaFlat,
                increased: Stat::MaxManaIncreased,
            },
            Stat::MovementSpeed => StatEvalKind::FlatIncreased {
                flat: Stat::MovementSpeedFlat,
                increased: Stat::MovementSpeedIncreased,
            },
            Stat::PickupRadius => StatEvalKind::FlatIncreased {
                flat: Stat::PickupRadiusFlat,
                increased: Stat::PickupRadiusIncreased,
            },
            Stat::CritChance => StatEvalKind::ClampedChance {
                flat: Stat::CritChanceFlat,
                increased: Stat::CritChanceIncreased,
            },

            _ => StatEvalKind::Sum,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatEvalKind {
    Sum,
    Product,
    FlatIncreased {
        flat: Stat,
        increased: Stat,
    },
    FlatIncreasedMore {
        flat: Stat,
        increased: Stat,
        more: Stat,
    },
    ClampedChance {
        flat: Stat,
        increased: Stat,
    },
}

impl StatEvalKind {
    pub fn dependencies(&self) -> Vec<Stat> {
        match self {
            StatEvalKind::Sum | StatEvalKind::Product => Vec::new(),
            StatEvalKind::FlatIncreased { flat, increased }
            | StatEvalKind::ClampedChance { flat, increased } => vec![*flat, *increased],
            StatEvalKind::FlatIncreasedMore {
                flat,
                increased,
                more,
            } => vec![*flat, *increased, *more],
        }
    }
}
