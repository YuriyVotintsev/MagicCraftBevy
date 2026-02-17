use bevy::prelude::*;
use rand::prelude::*;

use crate::stats::{DirtyStats, Modifiers};

use super::components::Affixes;
use super::registry::AffixRegistry;
use super::types::{Affix, AffixId};

pub fn apply_alteration(
    affixes: &mut Affixes,
    pool: &[AffixId],
    registry: &AffixRegistry,
    rng: &mut impl Rng,
) -> [bool; 6] {
    let mut rerolled = [false; 6];
    if pool.is_empty() {
        return rerolled;
    }
    let index = rng.random_range(0..6usize);
    rerolled[index] = true;
    let existing: Vec<AffixId> = affixes
        .affixes
        .iter()
        .enumerate()
        .filter(|(i, a)| *i != index && a.is_some())
        .filter_map(|(_, a)| a.as_ref().map(|a| a.affix_id))
        .collect();
    let available: Vec<&AffixId> = pool.iter().filter(|id| !existing.contains(id)).collect();
    if let Some(&&new_id) = available.choose(&mut *rng) {
        if let Some(def) = registry.get(new_id) {
            let tier = rng.random_range(0..=def.max_tier());
            let values = def.roll_values(tier, rng);
            affixes.affixes[index] = Some(Affix {
                affix_id: new_id,
                tier,
                values,
            });
        }
    }
    rerolled
}

pub fn apply_chaos(
    affixes: &mut Affixes,
    pool: &[AffixId],
    registry: &AffixRegistry,
    rng: &mut impl Rng,
) -> [bool; 6] {
    let mut rerolled = [false; 6];
    if pool.is_empty() {
        return rerolled;
    }
    let mut indices: Vec<usize> = (0..6).collect();
    indices.shuffle(&mut *rng);
    let chosen: Vec<usize> = indices.into_iter().take(3).collect();

    for &i in &chosen {
        rerolled[i] = true;
        affixes.affixes[i] = None;
    }

    for &i in &chosen {
        let existing: Vec<AffixId> = affixes
            .affixes
            .iter()
            .filter_map(|a| a.as_ref().map(|a| a.affix_id))
            .collect();
        let available: Vec<&AffixId> = pool.iter().filter(|id| !existing.contains(id)).collect();
        if let Some(&&new_id) = available.choose(&mut *rng) {
            if let Some(def) = registry.get(new_id) {
                let tier = rng.random_range(0..=def.max_tier());
                let values = def.roll_values(tier, rng);
                affixes.affixes[i] = Some(Affix {
                    affix_id: new_id,
                    tier,
                    values,
                });
            }
        }
    }
    rerolled
}

pub fn apply_augmentation(
    affixes: &mut Affixes,
    registry: &AffixRegistry,
    rng: &mut impl Rng,
) -> [bool; 6] {
    let mut rerolled = [false; 6];
    let upgradable: Vec<usize> = affixes
        .affixes
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            a.as_ref().and_then(|affix| {
                let def = registry.get(affix.affix_id)?;
                if affix.tier < def.max_tier() {
                    Some(i)
                } else {
                    None
                }
            })
        })
        .collect();

    if let Some(&index) = upgradable.choose(&mut *rng) {
        rerolled[index] = true;
        if let Some(affix) = &mut affixes.affixes[index] {
            let new_tier = affix.tier + 1;
            if let Some(def) = registry.get(affix.affix_id) {
                affix.values = def.roll_values(new_tier, rng);
            }
            affix.tier = new_tier;
        }
    }
    rerolled
}

pub fn sync_affix_modifiers(
    slot_entity: Entity,
    affixes: &Affixes,
    _affix_registry: &AffixRegistry,
    modifiers: &mut Modifiers,
    dirty: &mut DirtyStats,
) {
    let removed = modifiers.remove_by_source(slot_entity);
    dirty.mark_all(removed);

    for affix in affixes.affixes.iter().flatten() {
        for (stat, value) in &affix.values {
            modifiers.add(*stat, *value, Some(slot_entity));
            dirty.mark(*stat);
        }
    }
}
