use crate::stats::Modifiers;

use super::content::{write_pattern_contains, WriteEffect};
use super::data::RuneGrid;
use super::hex::HexCoord;

pub fn incoming_factor(target: HexCoord, grid: &RuneGrid) -> f32 {
    let mut factor = 1.0_f32;
    for (src_coord, src_rune) in grid.cells.iter() {
        if *src_coord == target {
            continue;
        }
        let Some(src_kind) = src_rune.kind else { continue };
        let Some(write) = src_kind.def().write else { continue };
        if !write_pattern_contains(&write, *src_coord, target) {
            continue;
        }
        match write.effect {
            WriteEffect::More { factor: f } => factor *= f,
        }
    }
    factor
}

pub fn add_grid_modifiers(grid: &RuneGrid, modifiers: &mut Modifiers) {
    for (coord, rune) in grid.cells.iter() {
        let Some(kind) = rune.kind else { continue };
        let (stat, mod_kind, base_value) = kind.def().base_effect;
        let factor = incoming_factor(*coord, grid);
        modifiers.add(stat, mod_kind, base_value * factor);
    }
}
