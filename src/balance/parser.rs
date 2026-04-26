use std::collections::HashMap;

use calamine::{Data, Range};

use crate::actors::MobKind;

use super::types::{
    Balance, Globals, MobCommonStats, MobsBalance, WaveDef, WavesConfig,
};

pub type BalanceError = String;

pub fn parse_balance(
    mobs: &Range<Data>,
    waves: &Range<Data>,
    globals: &Range<Data>,
) -> Result<Balance, BalanceError> {
    let mobs = parse_mobs(mobs).map_err(|e| format!("sheet Mobs: {e}"))?;
    let waves = parse_waves(waves).map_err(|e| format!("sheet Waves: {e}"))?;
    let globals = parse_globals(globals).map_err(|e| format!("sheet Globals: {e}"))?;
    Ok(Balance { mobs, waves, globals })
}

pub fn parse_mobs(range: &Range<Data>) -> Result<MobsBalance, BalanceError> {
    let headers = parse_headers(range)?;
    let c_id = required_col(&headers, "id")?;
    let c_hp = required_col(&headers, "hp")?;
    let c_damage = required_col(&headers, "damage")?;
    let c_speed = headers.get("speed").copied();
    let c_size = required_col(&headers, "size")?;
    let c_mass = headers.get("mass").copied();
    let c_attack_speed = headers.get("attack_speed").copied();

    let mut map: HashMap<MobKind, MobCommonStats> = HashMap::new();
    for (row_idx, row) in data_rows(range) {
        let id = cell_str(row.get(c_id))
            .ok_or_else(|| format!("row {row_idx}: empty id"))?;
        let kind = parse_mob_id(&id)
            .map_err(|e| format!("row {row_idx}: {e}"))?;
        if map.contains_key(&kind) {
            return Err(format!("row {row_idx}: duplicate mob id {id}"));
        }

        let hp = cell_f32(row.get(c_hp))
            .map_err(|e| format!("row {row_idx} hp: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: hp required"))?;
        let damage = cell_f32(row.get(c_damage))
            .map_err(|e| format!("row {row_idx} damage: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: damage required"))?;
        let size = cell_f32(row.get(c_size))
            .map_err(|e| format!("row {row_idx} size: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: size required"))?;
        let speed = match c_speed {
            Some(c) => cell_f32(row.get(c)).map_err(|e| format!("row {row_idx} speed: {e}"))?,
            None => None,
        };
        let mass = match c_mass {
            Some(c) => cell_f32(row.get(c)).map_err(|e| format!("row {row_idx} mass: {e}"))?,
            None => None,
        };
        let attack_speed = match c_attack_speed {
            Some(c) => cell_f32(row.get(c))
                .map_err(|e| format!("row {row_idx} attack_speed: {e}"))?,
            None => None,
        };

        map.insert(kind, MobCommonStats { hp, damage, speed, size, mass, attack_speed });
    }

    for kind in MobKind::iter() {
        if !map.contains_key(&kind) {
            return Err(format!("mob {} missing", kind.id()));
        }
    }

    Ok(MobsBalance {
        ghost: map.remove(&MobKind::Ghost).unwrap(),
        tower: map.remove(&MobKind::Tower).unwrap(),
        slime_small: map.remove(&MobKind::SlimeSmall).unwrap(),
        jumper: map.remove(&MobKind::Jumper).unwrap(),
        spinner: map.remove(&MobKind::Spinner).unwrap(),
    })
}

pub fn parse_waves(range: &Range<Data>) -> Result<WavesConfig, BalanceError> {
    let headers = parse_headers(range)?;
    let c_wave = required_col(&headers, "wave")?;
    let c_unlocks = required_col(&headers, "unlocks")?;
    let c_variety = required_col(&headers, "enemy_variety")?;
    let c_interval = required_col(&headers, "spawn_interval")?;
    let c_hp = required_col(&headers, "hp_multiplier")?;
    let c_dmg = required_col(&headers, "damage_multiplier")?;

    let mut rows: Vec<(u32, WaveDef, Option<MobKind>)> = Vec::new();
    for (row_idx, row) in data_rows(range) {
        let wave = cell_u32(row.get(c_wave))
            .map_err(|e| format!("row {row_idx} wave: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: wave required"))?;

        let unlocks = match cell_str(row.get(c_unlocks)) {
            Some(s) => Some(
                parse_mob_id(&s)
                    .map_err(|e| format!("row {row_idx} unlocks: {e}"))?,
            ),
            None => None,
        };

        let variety = cell_u32(row.get(c_variety))
            .map_err(|e| format!("row {row_idx} enemy_variety: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: enemy_variety required"))?;
        let interval = cell_f32(row.get(c_interval))
            .map_err(|e| format!("row {row_idx} spawn_interval: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: spawn_interval required"))?;
        let hp_m = cell_f32(row.get(c_hp))
            .map_err(|e| format!("row {row_idx} hp_multiplier: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: hp_multiplier required"))?;
        let dmg_m = cell_f32(row.get(c_dmg))
            .map_err(|e| format!("row {row_idx} damage_multiplier: {e}"))?
            .ok_or_else(|| format!("row {row_idx}: damage_multiplier required"))?;

        if variety == 0 {
            return Err(format!("wave {wave}: enemy_variety must be > 0"));
        }
        if interval <= 0.0 {
            return Err(format!("wave {wave}: spawn_interval must be > 0"));
        }
        if hp_m <= 0.0 {
            return Err(format!("wave {wave}: hp_multiplier must be > 0"));
        }
        if dmg_m <= 0.0 {
            return Err(format!("wave {wave}: damage_multiplier must be > 0"));
        }

        rows.push((
            wave,
            WaveDef {
                enemy_variety: variety,
                spawn_interval: interval,
                hp_multiplier: hp_m,
                damage_multiplier: dmg_m,
            },
            unlocks,
        ));
    }

    if rows.is_empty() {
        return Err("no waves".into());
    }
    rows.sort_by_key(|(w, _, _)| *w);
    for (i, (w, _, _)) in rows.iter().enumerate() {
        let expected = (i as u32) + 1;
        if *w != expected {
            return Err(format!(
                "wave numbers must be continuous starting at 1; got {w} at position {expected}"
            ));
        }
    }

    let mut mob_unlocks: HashMap<MobKind, u32> = HashMap::new();
    for (w, _, unlock) in &rows {
        if let Some(kind) = unlock {
            if mob_unlocks.contains_key(kind) {
                return Err(format!("mob {} unlocked more than once", kind.id()));
            }
            mob_unlocks.insert(*kind, *w);
        }
    }

    for (w, def, _) in &rows {
        let unlocked_count = mob_unlocks.values().filter(|u| **u <= *w).count() as u32;
        if def.enemy_variety > unlocked_count {
            bevy::log::warn!(
                "wave {w}: enemy_variety {} > {} mobs unlocked so far",
                def.enemy_variety,
                unlocked_count
            );
        }
    }

    let waves = rows.into_iter().map(|(_, d, _)| d).collect();
    Ok(WavesConfig { mob_unlocks, waves })
}

pub fn parse_globals(range: &Range<Data>) -> Result<Globals, BalanceError> {
    let headers = parse_headers(range)?;
    let c_key = required_col(&headers, "key")?;
    let c_value = required_col(&headers, "value")?;

    let mut map: HashMap<String, String> = HashMap::new();
    for (row_idx, row) in data_rows(range) {
        let key = cell_str(row.get(c_key))
            .ok_or_else(|| format!("row {row_idx}: empty key"))?;
        let value = cell_str(row.get(c_value))
            .ok_or_else(|| format!("row {row_idx}: value required for key {key}"))?;
        if map.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate key {key}"));
        }
    }

    let get_f32 = |k: &str| -> Result<f32, BalanceError> {
        map.get(k)
            .ok_or_else(|| format!("missing key {k}"))?
            .parse::<f32>()
            .map_err(|_| format!("key {k}: not a float"))
    };

    Ok(Globals {
        safe_spawn_radius: get_f32("safe_spawn_radius")?,
        arena_radius: get_f32("arena_radius")?,
    })
}

fn parse_headers(range: &Range<Data>) -> Result<HashMap<String, usize>, BalanceError> {
    let mut header_row = range.rows();
    let first = header_row.next().ok_or("empty sheet")?;
    let mut out: HashMap<String, usize> = HashMap::new();
    for (idx, cell) in first.iter().enumerate() {
        let Some(name) = cell_str(Some(cell)) else { continue };
        if name.starts_with('_') {
            continue;
        }
        if out.insert(name.clone(), idx).is_some() {
            return Err(format!("duplicate header {name}"));
        }
    }
    Ok(out)
}

fn required_col(headers: &HashMap<String, usize>, name: &str) -> Result<usize, BalanceError> {
    headers
        .get(name)
        .copied()
        .ok_or_else(|| format!("missing header {name}"))
}

fn data_rows<'a>(
    range: &'a Range<Data>,
) -> impl Iterator<Item = (usize, &'a [Data])> + 'a {
    range
        .rows()
        .enumerate()
        .skip(1)
        .filter(|(_, row)| row.iter().any(|c| !matches!(c, Data::Empty)))
        .map(|(i, r)| (i + 1, r))
}

fn cell_str(cell: Option<&Data>) -> Option<String> {
    match cell {
        Some(Data::String(s)) => {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t.to_string()) }
        }
        Some(Data::Int(i)) => Some(i.to_string()),
        Some(Data::Float(f)) => Some(f.to_string()),
        Some(Data::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

fn cell_f32(cell: Option<&Data>) -> Result<Option<f32>, BalanceError> {
    match cell {
        None | Some(Data::Empty) => Ok(None),
        Some(Data::Float(f)) => Ok(Some(*f as f32)),
        Some(Data::Int(i)) => Ok(Some(*i as f32)),
        Some(Data::String(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                t.parse::<f32>()
                    .map(Some)
                    .map_err(|_| format!("not a number: {s:?}"))
            }
        }
        Some(other) => Err(format!("unexpected cell {other:?}")),
    }
}

fn cell_u32(cell: Option<&Data>) -> Result<Option<u32>, BalanceError> {
    match cell {
        None | Some(Data::Empty) => Ok(None),
        Some(Data::Int(i)) => {
            if *i < 0 {
                Err(format!("negative int {i}"))
            } else {
                Ok(Some(*i as u32))
            }
        }
        Some(Data::Float(f)) => {
            if *f < 0.0 || f.fract() != 0.0 {
                Err(format!("not a non-negative integer: {f}"))
            } else {
                Ok(Some(*f as u32))
            }
        }
        Some(Data::String(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                t.parse::<u32>()
                    .map(Some)
                    .map_err(|_| format!("not a u32: {s:?}"))
            }
        }
        Some(other) => Err(format!("unexpected cell {other:?}")),
    }
}

fn parse_mob_id(s: &str) -> Result<MobKind, BalanceError> {
    MobKind::iter()
        .find(|k| k.id() == s)
        .ok_or_else(|| format!("unknown mob id: {s}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{open_workbook, Reader, Xlsx};

    fn load_real_workbook() -> (
        calamine::Range<Data>,
        calamine::Range<Data>,
        calamine::Range<Data>,
    ) {
        let path = "assets/balance.xlsx";
        let mut wb: Xlsx<_> = open_workbook(path).expect("open xlsx");
        (
            wb.worksheet_range("Mobs").expect("Mobs sheet"),
            wb.worksheet_range("Waves").expect("Waves sheet"),
            wb.worksheet_range("Globals").expect("Globals sheet"),
        )
    }

    #[test]
    fn happy_path_parses_real_xlsx() {
        let (mobs, waves, globals) = load_real_workbook();
        let bal = parse_balance(&mobs, &waves, &globals).expect("parse ok");

        assert!(bal.mobs.ghost.hp > 0.0);
        assert!(bal.mobs.ghost.speed.is_some());
        assert!(bal.mobs.tower.speed.is_none());
        assert!(bal.mobs.spinner.speed.is_some());
        assert!(bal.mobs.jumper.speed.is_some());
        assert!(bal.mobs.slime_small.speed.is_some());

        assert!(!bal.waves.waves.is_empty());
        let first = &bal.waves.waves[0];
        assert!(first.enemy_variety > 0);
        assert!(first.spawn_interval > 0.0);
        assert!(first.hp_multiplier > 0.0);
        assert!(first.damage_multiplier > 0.0);

        assert!(bal.globals.safe_spawn_radius > 0.0);
        assert!(bal.globals.arena_radius > 0.0);
    }

    #[test]
    fn cell_f32_empty_is_none() {
        assert_eq!(cell_f32(None).unwrap(), None);
        assert_eq!(cell_f32(Some(&Data::Empty)).unwrap(), None);
        assert_eq!(cell_f32(Some(&Data::Float(1.5))).unwrap(), Some(1.5));
        assert_eq!(cell_f32(Some(&Data::Int(3))).unwrap(), Some(3.0));
    }

    #[test]
    fn parse_mob_id_unknown_errors() {
        assert!(parse_mob_id("ghost").is_ok());
        assert!(parse_mob_id("no_such_mob").is_err());
    }

}

