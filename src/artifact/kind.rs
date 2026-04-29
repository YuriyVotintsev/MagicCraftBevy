use crate::stats::{ModifierKind, Stat};

use super::effect::{ArtifactEffect, DefensiveKind, ExoticKind, OnHitKind};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    BurningEdge,
    RunnersGrace,
    EagleEye,
    GlassCannon,
    ZephyrStep,
    WidePalm,
    KeenSight,
    ConcentratedRage,
    DeepReserves,

    SplitShot,
    PiercingArrow,
    BouncingBolt,
    SeekersHand,
    ConcussiveBlast,

    EmberBrand,
    FrostBite,
    BloodPact,
    RamHorn,
    ChainLightning,

    GuardianAegis,
    ShadowVeil,
    SpinedHusk,

    SunturretSeed,
    AetherPulse,
    CallOfBees,
}

pub struct ArtifactDef {
    pub name: &'static str,
    pub description: &'static str,
    pub effect: ArtifactEffect,
    pub replaces: &'static [ArtifactKind],
}

impl ArtifactKind {
    pub const ALL: &'static [ArtifactKind] = &[
        ArtifactKind::BurningEdge,
        ArtifactKind::RunnersGrace,
        ArtifactKind::EagleEye,
        ArtifactKind::GlassCannon,
        ArtifactKind::ZephyrStep,
        ArtifactKind::WidePalm,
        ArtifactKind::KeenSight,
        ArtifactKind::ConcentratedRage,
        ArtifactKind::DeepReserves,
        ArtifactKind::SplitShot,
        ArtifactKind::PiercingArrow,
        ArtifactKind::BouncingBolt,
        ArtifactKind::SeekersHand,
        ArtifactKind::ConcussiveBlast,
        ArtifactKind::EmberBrand,
        ArtifactKind::FrostBite,
        ArtifactKind::BloodPact,
        ArtifactKind::RamHorn,
        ArtifactKind::ChainLightning,
        ArtifactKind::GuardianAegis,
        ArtifactKind::ShadowVeil,
        ArtifactKind::SpinedHusk,
        ArtifactKind::SunturretSeed,
        ArtifactKind::AetherPulse,
        ArtifactKind::CallOfBees,
    ];

    pub fn def(self) -> ArtifactDef {
        match self {
            ArtifactKind::BurningEdge => ArtifactDef {
                name: "Burning Edge",
                description: "+25% damage",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::PhysicalDamage,
                    kind: ModifierKind::Increased,
                    value: 0.25,
                },
                replaces: &[],
            },
            ArtifactKind::RunnersGrace => ArtifactDef {
                name: "Runner's Grace",
                description: "+15% movement speed",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::MovementSpeed,
                    kind: ModifierKind::Increased,
                    value: 0.15,
                },
                replaces: &[],
            },
            ArtifactKind::EagleEye => ArtifactDef {
                name: "Eagle Eye",
                description: "+10% crit chance",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::CritChance,
                    kind: ModifierKind::Flat,
                    value: 0.10,
                },
                replaces: &[],
            },
            ArtifactKind::GlassCannon => ArtifactDef {
                name: "Glass Cannon",
                description: "+50% damage",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::PhysicalDamage,
                    kind: ModifierKind::More,
                    value: 0.50,
                },
                replaces: &[],
            },
            ArtifactKind::ZephyrStep => ArtifactDef {
                name: "Zephyr Step",
                description: "+30% movement speed",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::MovementSpeed,
                    kind: ModifierKind::More,
                    value: 0.30,
                },
                replaces: &[],
            },
            ArtifactKind::WidePalm => ArtifactDef {
                name: "Wide Palm",
                description: "+20% attack speed",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::AttackSpeed,
                    kind: ModifierKind::Increased,
                    value: 0.20,
                },
                replaces: &[],
            },
            ArtifactKind::KeenSight => ArtifactDef {
                name: "Keen Sight",
                description: "+0.5 crit multiplier",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::CritMultiplier,
                    kind: ModifierKind::Flat,
                    value: 0.5,
                },
                replaces: &[],
            },
            ArtifactKind::ConcentratedRage => ArtifactDef {
                name: "Concentrated Rage",
                description: "+30% attack speed",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::AttackSpeed,
                    kind: ModifierKind::More,
                    value: 0.30,
                },
                replaces: &[],
            },
            ArtifactKind::DeepReserves => ArtifactDef {
                name: "Deep Reserves",
                description: "+60 max life",
                effect: ArtifactEffect::StatMod {
                    stat: Stat::MaxLife,
                    kind: ModifierKind::Flat,
                    value: 60.0,
                },
                replaces: &[],
            },

            ArtifactKind::SplitShot => ArtifactDef {
                name: "Split Shot",
                description: "+1 projectile",
                effect: ArtifactEffect::Multishot { extra: 1 },
                replaces: &[],
            },
            ArtifactKind::PiercingArrow => ArtifactDef {
                name: "Piercing Arrow",
                description: "Projectiles pierce 2 enemies",
                effect: ArtifactEffect::Pierce { extra: 2 },
                replaces: &[],
            },
            ArtifactKind::BouncingBolt => ArtifactDef {
                name: "Bouncing Bolt",
                description: "Projectiles ricochet 2 times",
                effect: ArtifactEffect::Ricochet { count: 2 },
                replaces: &[],
            },
            ArtifactKind::SeekersHand => ArtifactDef {
                name: "Seeker's Hand",
                description: "Projectiles home in on enemies",
                effect: ArtifactEffect::Homing { strength: 1.0 },
                replaces: &[],
            },
            ArtifactKind::ConcussiveBlast => ArtifactDef {
                name: "Concussive Blast",
                description: "Projectiles explode on impact",
                effect: ArtifactEffect::Splash { radius: 120.0 },
                replaces: &[],
            },

            ArtifactKind::EmberBrand => ArtifactDef {
                name: "Ember Brand",
                description: "Hits ignite enemies",
                effect: ArtifactEffect::OnHit(OnHitKind::Burn {
                    dps: 4.0,
                    duration: 3.0,
                }),
                replaces: &[],
            },
            ArtifactKind::FrostBite => ArtifactDef {
                name: "Frost Bite",
                description: "Chance to freeze on hit",
                effect: ArtifactEffect::OnHit(OnHitKind::Freeze {
                    chance: 0.2,
                    duration: 1.5,
                }),
                replaces: &[],
            },
            ArtifactKind::BloodPact => ArtifactDef {
                name: "Blood Pact",
                description: "Steal 8% of damage as life",
                effect: ArtifactEffect::OnHit(OnHitKind::Lifesteal { pct: 0.08 }),
                replaces: &[],
            },
            ArtifactKind::RamHorn => ArtifactDef {
                name: "Ram Horn",
                description: "Hits knock enemies back",
                effect: ArtifactEffect::OnHit(OnHitKind::Knockback { force: 250.0 }),
                replaces: &[],
            },
            ArtifactKind::ChainLightning => ArtifactDef {
                name: "Chain Lightning",
                description: "Hits arc to nearby enemies",
                effect: ArtifactEffect::OnHit(OnHitKind::Chain { count: 2 }),
                replaces: &[],
            },

            ArtifactKind::GuardianAegis => ArtifactDef {
                name: "Guardian Aegis",
                description: "Recharging shield (15)",
                effect: ArtifactEffect::Defensive(DefensiveKind::Shield {
                    max_block: 15.0,
                    recharge: 5.0,
                }),
                replaces: &[],
            },
            ArtifactKind::ShadowVeil => ArtifactDef {
                name: "Shadow Veil",
                description: "20% chance to dodge",
                effect: ArtifactEffect::Defensive(DefensiveKind::Dodge { chance: 0.20 }),
                replaces: &[],
            },
            ArtifactKind::SpinedHusk => ArtifactDef {
                name: "Spined Husk",
                description: "Reflect 25% damage taken",
                effect: ArtifactEffect::Defensive(DefensiveKind::Thorns { reflect_pct: 0.25 }),
                replaces: &[],
            },
            ArtifactKind::SunturretSeed => ArtifactDef {
                name: "Sunturret Seed",
                description: "A turret follows you",
                effect: ArtifactEffect::Exotic(ExoticKind::Turret {
                    fire_interval: 1.2,
                    damage_pct: 0.6,
                }),
                replaces: &[],
            },
            ArtifactKind::AetherPulse => ArtifactDef {
                name: "Aether Pulse",
                description: "Periodic AOE around you",
                effect: ArtifactEffect::Exotic(ExoticKind::PeriodicAoe {
                    interval: 2.0,
                    radius: 200.0,
                    damage_pct: 0.5,
                }),
                replaces: &[],
            },
            ArtifactKind::CallOfBees => ArtifactDef {
                name: "Call of Bees",
                description: "Five orbiting bees",
                effect: ArtifactEffect::Exotic(ExoticKind::OrbitingOrbs {
                    count: 5,
                    radius: 140.0,
                    damage: 3.0,
                }),
                replaces: &[],
            },
        }
    }
}
