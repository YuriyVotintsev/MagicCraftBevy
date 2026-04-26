use rand::Rng;

use super::inventory::ArtifactInventory;
use super::kind::ArtifactKind;

pub fn tier_weights(wave: u32) -> [u32; 4] {
    let w = wave.min(30) as i32;
    let common = (70 - 2 * w).max(15) as u32;
    let rare = (25 + (w / 2)).clamp(0, 35) as u32;
    let epic = (w.saturating_sub(2)).clamp(0, 35) as u32;
    let legendary = ((w - 5).max(0) / 2).clamp(0, 20) as u32;
    [common, rare, epic, legendary]
}

pub fn roll_artifact(
    wave: u32,
    inv: &ArtifactInventory,
    rng: &mut impl Rng,
) -> Option<ArtifactKind> {
    pick(wave, inv, None, rng)
}

pub fn roll_artifact_excluding(
    wave: u32,
    inv: &ArtifactInventory,
    skip: ArtifactKind,
    rng: &mut impl Rng,
) -> Option<ArtifactKind> {
    pick(wave, inv, Some(skip), rng)
}

fn pick(
    wave: u32,
    inv: &ArtifactInventory,
    skip: Option<ArtifactKind>,
    rng: &mut impl Rng,
) -> Option<ArtifactKind> {
    let weights = tier_weights(wave);

    let by_tier: [Vec<ArtifactKind>; 4] = {
        let mut buckets: [Vec<ArtifactKind>; 4] = Default::default();
        for &k in ArtifactKind::ALL {
            if inv.contains(k) {
                continue;
            }
            if Some(k) == skip {
                continue;
            }
            buckets[k.def().tier.index()].push(k);
        }
        buckets
    };

    let effective: [u32; 4] = {
        let mut w = [0u32; 4];
        for (i, list) in by_tier.iter().enumerate() {
            if !list.is_empty() {
                w[i] = weights[i];
            }
        }
        w
    };

    let total: u32 = effective.iter().sum();
    if total == 0 {
        return None;
    }

    let mut roll = rng.random_range(0..total);
    let mut tier_idx = 0usize;
    for (i, &w) in effective.iter().enumerate() {
        if roll < w {
            tier_idx = i;
            break;
        }
        roll -= w;
    }

    let bucket = &by_tier[tier_idx];
    if bucket.is_empty() {
        return None;
    }
    Some(bucket[rng.random_range(0..bucket.len())])
}
