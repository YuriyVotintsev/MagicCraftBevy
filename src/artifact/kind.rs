use crate::stats::{ModifierKind, Stat};

use super::effect::{ArtifactEffect, DefensiveKind, ExoticKind, OnHitKind};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tier {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Tier {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn palette_key(self) -> &'static str {
        match self {
            Tier::Common => "ui_artifact_common",
            Tier::Rare => "ui_artifact_rare",
            Tier::Epic => "ui_artifact_epic",
            Tier::Legendary => "ui_artifact_legendary",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    SharpenedCore,
    BurningEdge,
    IronHide,
    RunnersGrace,
    EagleEye,
    GlassCannon,
    HeartOfOak,
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
    DodgeMantle,

    SunturretSeed,
    OrbitingMotes,
    AetherPulse,
    CallOfBees,
}

pub struct ArtifactDef {
    pub name: &'static str,
    pub description: &'static str,
    pub tier: Tier,
    pub effect: ArtifactEffect,
    pub replaces: &'static [ArtifactKind],
}

impl ArtifactKind {
    pub const ALL: &'static [ArtifactKind] = &[
        ArtifactKind::SharpenedCore,
        ArtifactKind::BurningEdge,
        ArtifactKind::IronHide,
        ArtifactKind::RunnersGrace,
        ArtifactKind::EagleEye,
        ArtifactKind::GlassCannon,
        ArtifactKind::HeartOfOak,
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
        ArtifactKind::DodgeMantle,
        ArtifactKind::SunturretSeed,
        ArtifactKind::OrbitingMotes,
        ArtifactKind::AetherPulse,
        ArtifactKind::CallOfBees,
    ];

    pub fn def(self) -> ArtifactDef {
        match self {
            ArtifactKind::SharpenedCore => ArtifactDef {
                name: "Sharpened Core",
                description: "+15% damage",
                tier: Tier::Common,
                effect: ArtifactEffect::StatMod {
                    stat: Stat::PhysicalDamage,
                    kind: ModifierKind::Increased,
                    value: 0.15,
                },
                replaces: &[],
            },
            ArtifactKind::BurningEdge => ArtifactDef {
                name: "Burning Edge",
                description: "+25% damage",
                tier: Tier::Rare,
                effect: ArtifactEffect::StatMod {
                    stat: Stat::PhysicalDamage,
                    kind: ModifierKind::Increased,
                    value: 0.25,
                },
                replaces: &[],
            },
            ArtifactKind::IronHide => ArtifactDef {
                name: "Iron Hide",
                description: "+10 max life",
                tier: Tier::Common,
                effect: ArtifactEffect::StatMod {
                    stat: Stat::MaxLife,
                    kind: ModifierKind::Flat,
                    value: 10.0,
                },
                replaces: &[],
            },
            ArtifactKind::RunnersGrace => ArtifactDef {
                name: "Runner's Grace",
                description: "+15% movement speed",
                tier: Tier::Common,
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
                tier: Tier::Rare,
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
                tier: Tier::Epic,
                effect: ArtifactEffect::StatMod {
                    stat: Stat::PhysicalDamage,
                    kind: ModifierKind::More,
                    value: 0.50,
                },
                replaces: &[],
            },
            ArtifactKind::HeartOfOak => ArtifactDef {
                name: "Heart of Oak",
                description: "+30 max life",
                tier: Tier::Rare,
                effect: ArtifactEffect::StatMod {
                    stat: Stat::MaxLife,
                    kind: ModifierKind::Flat,
                    value: 30.0,
                },
                replaces: &[],
            },
            ArtifactKind::ZephyrStep => ArtifactDef {
                name: "Zephyr Step",
                description: "+30% movement speed",
                tier: Tier::Epic,
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
                tier: Tier::Rare,
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
                tier: Tier::Rare,
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
                tier: Tier::Epic,
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
                tier: Tier::Epic,
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
                tier: Tier::Rare,
                effect: ArtifactEffect::Multishot { extra: 1 },
                replaces: &[],
            },
            ArtifactKind::PiercingArrow => ArtifactDef {
                name: "Piercing Arrow",
                description: "Projectiles pierce 2 enemies",
                tier: Tier::Rare,
                effect: ArtifactEffect::Pierce { extra: 2 },
                replaces: &[],
            },
            ArtifactKind::BouncingBolt => ArtifactDef {
                name: "Bouncing Bolt",
                description: "Projectiles ricochet 2 times",
                tier: Tier::Rare,
                effect: ArtifactEffect::Ricochet { count: 2 },
                replaces: &[],
            },
            ArtifactKind::SeekersHand => ArtifactDef {
                name: "Seeker's Hand",
                description: "Projectiles home in on enemies",
                tier: Tier::Epic,
                effect: ArtifactEffect::Homing { strength: 1.0 },
                replaces: &[],
            },
            ArtifactKind::ConcussiveBlast => ArtifactDef {
                name: "Concussive Blast",
                description: "Projectiles explode on impact",
                tier: Tier::Epic,
                effect: ArtifactEffect::Splash { radius: 120.0 },
                replaces: &[],
            },

            ArtifactKind::EmberBrand => ArtifactDef {
                name: "Ember Brand",
                description: "Hits ignite enemies",
                tier: Tier::Rare,
                effect: ArtifactEffect::OnHit(OnHitKind::Burn {
                    dps: 4.0,
                    duration: 3.0,
                }),
                replaces: &[],
            },
            ArtifactKind::FrostBite => ArtifactDef {
                name: "Frost Bite",
                description: "Chance to freeze on hit",
                tier: Tier::Rare,
                effect: ArtifactEffect::OnHit(OnHitKind::Freeze {
                    chance: 0.2,
                    duration: 1.5,
                }),
                replaces: &[],
            },
            ArtifactKind::BloodPact => ArtifactDef {
                name: "Blood Pact",
                description: "Steal 8% of damage as life",
                tier: Tier::Epic,
                effect: ArtifactEffect::OnHit(OnHitKind::Lifesteal { pct: 0.08 }),
                replaces: &[],
            },
            ArtifactKind::RamHorn => ArtifactDef {
                name: "Ram Horn",
                description: "Hits knock enemies back",
                tier: Tier::Common,
                effect: ArtifactEffect::OnHit(OnHitKind::Knockback { force: 250.0 }),
                replaces: &[],
            },
            ArtifactKind::ChainLightning => ArtifactDef {
                name: "Chain Lightning",
                description: "Hits arc to nearby enemies",
                tier: Tier::Legendary,
                effect: ArtifactEffect::OnHit(OnHitKind::Chain { count: 2 }),
                replaces: &[],
            },

            ArtifactKind::GuardianAegis => ArtifactDef {
                name: "Guardian Aegis",
                description: "Recharging shield (15)",
                tier: Tier::Epic,
                effect: ArtifactEffect::Defensive(DefensiveKind::Shield {
                    max_block: 15.0,
                    recharge: 5.0,
                }),
                replaces: &[],
            },
            ArtifactKind::ShadowVeil => ArtifactDef {
                name: "Shadow Veil",
                description: "20% chance to dodge",
                tier: Tier::Rare,
                effect: ArtifactEffect::Defensive(DefensiveKind::Dodge { chance: 0.20 }),
                replaces: &[],
            },
            ArtifactKind::SpinedHusk => ArtifactDef {
                name: "Spined Husk",
                description: "Reflect 25% damage taken",
                tier: Tier::Epic,
                effect: ArtifactEffect::Defensive(DefensiveKind::Thorns { reflect_pct: 0.25 }),
                replaces: &[],
            },
            ArtifactKind::DodgeMantle => ArtifactDef {
                name: "Dodge Mantle",
                description: "+10% chance to dodge",
                tier: Tier::Common,
                effect: ArtifactEffect::Defensive(DefensiveKind::Dodge { chance: 0.10 }),
                replaces: &[],
            },

            ArtifactKind::SunturretSeed => ArtifactDef {
                name: "Sunturret Seed",
                description: "A turret follows you",
                tier: Tier::Legendary,
                effect: ArtifactEffect::Exotic(ExoticKind::Turret {
                    fire_interval: 1.2,
                    damage_pct: 0.6,
                }),
                replaces: &[],
            },
            ArtifactKind::OrbitingMotes => ArtifactDef {
                name: "Orbiting Motes",
                description: "Three motes orbit you",
                tier: Tier::Epic,
                effect: ArtifactEffect::Exotic(ExoticKind::OrbitingOrbs {
                    count: 3,
                    radius: 110.0,
                    damage: 4.0,
                }),
                replaces: &[],
            },
            ArtifactKind::AetherPulse => ArtifactDef {
                name: "Aether Pulse",
                description: "Periodic AOE around you",
                tier: Tier::Legendary,
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
                tier: Tier::Legendary,
                effect: ArtifactEffect::Exotic(ExoticKind::OrbitingOrbs {
                    count: 5,
                    radius: 140.0,
                    damage: 3.0,
                }),
                replaces: &[ArtifactKind::OrbitingMotes],
            },
        }
    }
}
