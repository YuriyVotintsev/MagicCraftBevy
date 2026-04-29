use rand::Rng;

use super::effect::ArtifactEffect;
use super::inventory::ArtifactInventory;
use super::kind::ArtifactKind;

pub fn is_stat_mod(k: ArtifactKind) -> bool {
    matches!(k.def().effect, ArtifactEffect::StatMod { .. })
}

pub fn roll_artifact(
    inv: &ArtifactInventory,
    prev_accepted: Option<ArtifactKind>,
    rng: &mut impl Rng,
) -> Option<ArtifactKind> {
    pick(inv, prev_accepted, None, rng)
}

pub fn roll_artifact_excluding(
    inv: &ArtifactInventory,
    prev_accepted: Option<ArtifactKind>,
    skip: ArtifactKind,
    rng: &mut impl Rng,
) -> Option<ArtifactKind> {
    pick(inv, prev_accepted, Some(skip), rng)
}

fn pick(
    inv: &ArtifactInventory,
    prev_accepted: Option<ArtifactKind>,
    skip: Option<ArtifactKind>,
    rng: &mut impl Rng,
) -> Option<ArtifactKind> {
    let block_stat_mod = prev_accepted.map(is_stat_mod).unwrap_or(false);
    let candidates: Vec<ArtifactKind> = ArtifactKind::ALL
        .iter()
        .copied()
        .filter(|&k| !inv.contains(k))
        .filter(|&k| Some(k) != skip)
        .filter(|&k| !(block_stat_mod && is_stat_mod(k)))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    Some(candidates[rng.random_range(0..candidates.len())])
}
