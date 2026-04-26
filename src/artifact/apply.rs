use bevy::prelude::*;

use crate::actors::components::Health;
use crate::actors::Player;
use crate::stats::{
    ComputedStats, DirtyStats, ModifierKind, Modifiers, Stat, StatCalculators,
};

use super::effect::{ArtifactEffect, DefensiveKind, OnHitKind};
use super::exotic::{attach_exotic, ExoticHelper};
use super::inventory::ArtifactInventory;
use super::kind::ArtifactKind;

pub const PLAYER_BASE_STATS: &[(Stat, ModifierKind, f32)] = &[
    (Stat::MaxLife, ModifierKind::Flat, 20.0),
    (Stat::MovementSpeed, ModifierKind::Flat, 550.0),
    (Stat::PhysicalDamage, ModifierKind::Flat, 1.0),
    (Stat::CritChance, ModifierKind::Flat, 0.05),
    (Stat::CritMultiplier, ModifierKind::Flat, 1.5),
    (Stat::AttackSpeed, ModifierKind::Flat, 1.0),
];

#[derive(Message)]
pub struct RebuildPlayerStateEvent;

pub fn register(app: &mut App) {
    app.add_message::<RebuildPlayerStateEvent>()
        .add_systems(Update, rebuild_player_state);
}

pub fn apply_artifact_to_modifiers(kind: ArtifactKind, m: &mut Modifiers) {
    match kind.def().effect {
        ArtifactEffect::StatMod { stat, kind: mk, value } => m.add(stat, mk, value),
        ArtifactEffect::Multishot { extra } => {
            m.add(Stat::ProjectileCount, ModifierKind::Flat, extra as f32)
        }
        ArtifactEffect::Pierce { extra } => {
            m.add(Stat::Pierce, ModifierKind::Flat, extra as f32)
        }
        ArtifactEffect::Ricochet { count } => {
            m.add(Stat::Ricochet, ModifierKind::Flat, count as f32)
        }
        ArtifactEffect::Homing { strength } => {
            m.add(Stat::HomingStrength, ModifierKind::Flat, strength)
        }
        ArtifactEffect::Splash { radius, .. } => {
            m.add(Stat::SplashRadius, ModifierKind::Flat, radius)
        }
        ArtifactEffect::OnHit(OnHitKind::Burn { dps, duration }) => {
            m.add(Stat::BurnDPS, ModifierKind::Flat, dps);
            m.add(Stat::BurnDuration, ModifierKind::Flat, duration);
        }
        ArtifactEffect::OnHit(OnHitKind::Freeze {
            chance, duration, ..
        }) => {
            m.add(Stat::FreezeChance, ModifierKind::Flat, chance);
            m.add(Stat::FreezeDuration, ModifierKind::Flat, duration);
        }
        ArtifactEffect::OnHit(OnHitKind::Lifesteal { pct }) => {
            m.add(Stat::Lifesteal, ModifierKind::Flat, pct)
        }
        ArtifactEffect::OnHit(OnHitKind::Knockback { force }) => {
            m.add(Stat::KnockbackForce, ModifierKind::Flat, force)
        }
        ArtifactEffect::OnHit(OnHitKind::Chain { count, .. }) => {
            m.add(Stat::ChainCount, ModifierKind::Flat, count as f32)
        }
        ArtifactEffect::Defensive(DefensiveKind::Dodge { chance }) => {
            m.add(Stat::DodgeChance, ModifierKind::Flat, chance)
        }
        ArtifactEffect::Defensive(DefensiveKind::Thorns { reflect_pct }) => {
            m.add(Stat::Thorns, ModifierKind::Flat, reflect_pct)
        }
        ArtifactEffect::Defensive(DefensiveKind::Shield {
            max_block,
            recharge,
        }) => {
            m.add(Stat::ShieldMaxBlock, ModifierKind::Flat, max_block);
            m.add(Stat::ShieldRecharge, ModifierKind::Flat, recharge);
        }
        ArtifactEffect::Exotic(_) => {}
    }
}

pub fn build_player_modifiers(inv: &ArtifactInventory) -> Modifiers {
    let mut mods = Modifiers::new();
    for &(s, k, v) in PLAYER_BASE_STATS {
        mods.add(s, k, v);
    }
    for kind in inv.active() {
        apply_artifact_to_modifiers(kind, &mut mods);
    }
    mods
}

pub fn apply_inventory_to_player(
    commands: &mut Commands,
    player: Entity,
    inv: &ArtifactInventory,
    calculators: &StatCalculators,
) -> (Modifiers, ComputedStats) {
    let mods = build_player_modifiers(inv);
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::iter());
    calculators.recalculate(&mods, &mut computed, &mut dirty);

    for kind in inv.active() {
        if let ArtifactEffect::Exotic(e) = kind.def().effect {
            attach_exotic(commands, player, e);
        }
    }
    (mods, computed)
}

fn rebuild_player_state(
    mut commands: Commands,
    mut ev: MessageReader<RebuildPlayerStateEvent>,
    inventory: Res<ArtifactInventory>,
    calculators: Res<StatCalculators>,
    mut player_q: Query<(Entity, &ComputedStats, Option<&mut Health>), With<Player>>,
    helper_q: Query<Entity, With<ExoticHelper>>,
) {
    if ev.read().last().is_none() {
        return;
    }
    let Ok((player, old_computed, mut maybe_health)) = player_q.single_mut() else {
        return;
    };
    let old_max_life = old_computed.final_of(Stat::MaxLife);
    for e in &helper_q {
        if let Ok(mut ec) = commands.get_entity(e) {
            ec.despawn();
        }
    }
    let (mods, computed) =
        apply_inventory_to_player(&mut commands, player, &inventory, &calculators);
    let new_max_life = computed.final_of(Stat::MaxLife);
    let max_life_gain = (new_max_life - old_max_life).max(0.0);
    if max_life_gain > 0.0 {
        if let Some(health) = maybe_health.as_mut() {
            health.current = (health.current + max_life_gain).min(new_max_life);
        }
    }
    let mut dirty = DirtyStats::default();
    dirty.mark_all(Stat::iter());
    commands
        .entity(player)
        .insert((mods, computed, dirty))
        .remove::<crate::artifact::exotic::PeriodicAoe>();
    for kind in inventory.active() {
        if let ArtifactEffect::Exotic(crate::artifact::ExoticKind::PeriodicAoe {
            interval,
            radius,
            damage_pct,
        }) = kind.def().effect
        {
            commands.entity(player).insert(crate::artifact::exotic::PeriodicAoe {
                interval,
                radius,
                damage_pct,
                cooldown: interval,
            });
        }
    }
}
