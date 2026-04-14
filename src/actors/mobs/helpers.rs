use super::super::components::SpriteColor;
use crate::palette;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, Stat, StatCalculators};

pub(super) fn enemy_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy").unwrap_or((1.0, 1.0, 1.0));
    let flash = palette::flash_lookup("enemy");
    SpriteColor { r, g, b, a: 1.0, flash }
}

pub(super) fn enemy_ability_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
    let flash = palette::flash_lookup("enemy_ability");
    SpriteColor { r, g, b, a: 1.0, flash }
}

pub(super) fn compute_stats(
    calculators: &StatCalculators,
    base_stats: &[(Stat, f32)],
    extra_modifiers: &[(Stat, f32)],
) -> (Modifiers, DirtyStats, ComputedStats) {
    let mut modifiers = Modifiers::new();
    for &(stat, value) in base_stats {
        modifiers.add(stat, value);
    }
    for &(stat, value) in extra_modifiers {
        modifiers.add(stat, value);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::ALL.iter().copied());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    (modifiers, dirty, computed)
}

pub(super) fn current_max_life(computed: &ComputedStats) -> f32 {
    computed.get(Stat::MaxLife)
}
