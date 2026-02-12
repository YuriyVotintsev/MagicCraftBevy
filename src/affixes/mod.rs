use std::collections::HashMap;

use bevy::prelude::*;
use rand::prelude::*;

use crate::player::{Player, SpellSlot};
use crate::stats::{DirtyStats, Modifiers, StatId};
use crate::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AffixId(pub u32);

pub struct AffixDef {
    pub name: String,
    pub stat: StatId,
    pub tiers: Vec<f32>,
}

impl AffixDef {
    pub fn format_value(&self, tier: usize) -> String {
        let value = self.tiers.get(tier).copied().unwrap_or(0.0);
        let display = if self.name.contains('%') {
            format!("{}", (value * 100.0).round() as i32)
        } else {
            format!("{}", value.round() as i32)
        };
        self.name.replace("{}", &display)
    }

    pub fn format_display(&self, tier: usize) -> String {
        format!("{} [T{}]", self.format_value(tier), tier + 1)
    }

    pub fn max_tier(&self) -> usize {
        self.tiers.len().saturating_sub(1)
    }
}

#[derive(serde::Deserialize)]
pub struct AffixTierRaw {
    pub value: f32,
}

#[derive(serde::Deserialize)]
pub struct AffixDefRaw {
    pub id: String,
    pub name: String,
    pub stat: String,
    pub tiers: Vec<AffixTierRaw>,
}

#[derive(Debug, Clone, Copy)]
pub struct Affix {
    pub affix_id: AffixId,
    pub tier: usize,
}

#[derive(Component, Clone, Default)]
pub struct Affixes {
    pub affixes: [Option<Affix>; 6],
}

#[derive(Component)]
pub struct SpellSlotTag(pub SpellSlot);

#[derive(Component)]
pub struct SlotOwner(pub Entity);

#[derive(Resource, Default)]
pub struct AffixRegistry {
    affixes: Vec<AffixDef>,
    pools: HashMap<SpellSlot, Vec<AffixId>>,
}

impl AffixRegistry {
    pub fn register(&mut self, def: AffixDef, slot: SpellSlot) -> AffixId {
        let id = AffixId(self.affixes.len() as u32);
        self.affixes.push(def);
        self.pools.entry(slot).or_default().push(id);
        id
    }

    pub fn get(&self, id: AffixId) -> Option<&AffixDef> {
        self.affixes.get(id.0 as usize)
    }

    pub fn pool(&self, slot: SpellSlot) -> &[AffixId] {
        self.pools.get(&slot).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrbId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum OrbBehavior {
    Alteration,
    Chaos,
    Augmentation,
}

pub struct OrbDef {
    pub name: String,
    pub description: String,
    pub price: u32,
    pub behavior: OrbBehavior,
}

#[derive(serde::Deserialize)]
pub struct OrbDefRaw {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: u32,
    pub orb_type: OrbBehavior,
}

#[derive(Resource, Default)]
pub struct OrbRegistry {
    orbs: Vec<OrbDef>,
    name_to_id: HashMap<String, OrbId>,
}

impl OrbRegistry {
    pub fn register(&mut self, id_str: &str, def: OrbDef) -> OrbId {
        let id = OrbId(self.orbs.len() as u32);
        self.name_to_id.insert(id_str.to_string(), id);
        self.orbs.push(def);
        id
    }

    pub fn get(&self, id: OrbId) -> Option<&OrbDef> {
        self.orbs.get(id.0 as usize)
    }

    pub fn all_ids(&self) -> Vec<OrbId> {
        (0..self.orbs.len() as u32).map(OrbId).collect()
    }
}

#[derive(Resource, Default)]
pub enum OrbFlowState {
    #[default]
    None,
    SelectSlot {
        orb_id: OrbId,
    },
    Preview {
        orb_id: OrbId,
        slot_entity: Entity,
        slot_type: SpellSlot,
        original: Affixes,
        preview: Affixes,
    },
}

pub fn apply_alteration(affixes: &mut Affixes, pool: &[AffixId], rng: &mut impl Rng) {
    if pool.is_empty() {
        return;
    }
    let index = rng.random_range(0..6usize);
    let existing: Vec<AffixId> = affixes
        .affixes
        .iter()
        .enumerate()
        .filter(|(i, a)| *i != index && a.is_some())
        .filter_map(|(_, a)| a.map(|a| a.affix_id))
        .collect();
    let available: Vec<&AffixId> = pool.iter().filter(|id| !existing.contains(id)).collect();
    if let Some(&&new_id) = available.choose(&mut *rng) {
        affixes.affixes[index] = Some(Affix {
            affix_id: new_id,
            tier: 0,
        });
    }
}

pub fn apply_chaos(affixes: &mut Affixes, pool: &[AffixId], rng: &mut impl Rng) {
    if pool.is_empty() {
        return;
    }
    let mut indices: Vec<usize> = (0..6).collect();
    indices.shuffle(&mut *rng);
    let chosen: Vec<usize> = indices.into_iter().take(3).collect();

    for &i in &chosen {
        affixes.affixes[i] = None;
    }

    for &i in &chosen {
        let existing: Vec<AffixId> = affixes
            .affixes
            .iter()
            .filter_map(|a| a.map(|a| a.affix_id))
            .collect();
        let available: Vec<&AffixId> = pool.iter().filter(|id| !existing.contains(id)).collect();
        if let Some(&&new_id) = available.choose(&mut *rng) {
            affixes.affixes[i] = Some(Affix {
                affix_id: new_id,
                tier: 0,
            });
        }
    }
}

pub fn apply_augmentation(affixes: &mut Affixes, registry: &AffixRegistry, rng: &mut impl Rng) {
    let upgradable: Vec<usize> = affixes
        .affixes
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            a.and_then(|affix| {
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
        if let Some(affix) = &mut affixes.affixes[index] {
            affix.tier += 1;
        }
    }
}

pub fn sync_affix_modifiers(
    slot_entity: Entity,
    affixes: &Affixes,
    affix_registry: &AffixRegistry,
    modifiers: &mut Modifiers,
    dirty: &mut DirtyStats,
) {
    let removed = modifiers.remove_by_source(slot_entity);
    dirty.mark_all(removed);

    for affix in affixes.affixes.iter().flatten() {
        if let Some(def) = affix_registry.get(affix.affix_id) {
            if let Some(&value) = def.tiers.get(affix.tier) {
                modifiers.add(def.stat, value, Some(slot_entity));
                dirty.mark(def.stat);
            }
        }
    }
}

fn spawn_spell_slots(mut commands: Commands, player_query: Query<Entity, Added<Player>>) {
    for player_entity in &player_query {
        for slot in [SpellSlot::Active, SpellSlot::Passive, SpellSlot::Defensive] {
            commands.spawn((
                SpellSlotTag(slot),
                Affixes::default(),
                SlotOwner(player_entity),
                DespawnOnExit(GameState::Playing),
            ));
        }
    }
}

pub struct AffixPlugin;

impl Plugin for AffixPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AffixRegistry>()
            .init_resource::<OrbRegistry>()
            .init_resource::<OrbFlowState>()
            .add_systems(
                Update,
                spawn_spell_slots.run_if(in_state(GameState::Playing)),
            );
    }
}
