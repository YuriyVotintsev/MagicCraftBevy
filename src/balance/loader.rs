use std::io::{Read, Seek};

use bevy::prelude::*;
use calamine::{Reader, Xlsx};

use super::parser::{parse_balance, BalanceError};
use super::types::Balance;

#[cfg(any(not(feature = "dev"), target_arch = "wasm32"))]
const BALANCE_XLSX: &[u8] = include_bytes!("../../assets/balance.xlsx");

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
const XLSX_PATH: &str = "assets/balance.xlsx";

pub fn load_balance() -> Result<Balance, BalanceError> {
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    {
        use calamine::open_workbook;
        let mut wb: Xlsx<_> =
            open_workbook(XLSX_PATH).map_err(|e| format!("opening {XLSX_PATH}: {e}"))?;
        load_from_workbook(&mut wb)
    }
    #[cfg(any(not(feature = "dev"), target_arch = "wasm32"))]
    {
        let cursor = std::io::Cursor::new(BALANCE_XLSX);
        let mut wb = Xlsx::new(cursor).map_err(|e| format!("reading embedded xlsx: {e}"))?;
        load_from_workbook(&mut wb)
    }
}

fn load_from_workbook<R: Read + Seek>(wb: &mut Xlsx<R>) -> Result<Balance, BalanceError> {
    let mobs = wb
        .worksheet_range("Mobs")
        .map_err(|e| format!("sheet Mobs: {e}"))?;
    let waves = wb
        .worksheet_range("Waves")
        .map_err(|e| format!("sheet Waves: {e}"))?;
    let runes = wb
        .worksheet_range("Runes")
        .map_err(|e| format!("sheet Runes: {e}"))?;
    let globals = wb
        .worksheet_range("Globals")
        .map_err(|e| format!("sheet Globals: {e}"))?;
    parse_balance(&mobs, &waves, &runes, &globals)
}

pub fn setup_balance(mut commands: Commands) {
    match load_balance() {
        Ok(balance) => install_balance(&mut commands, balance),
        Err(e) => {
            #[cfg(feature = "dev")]
            panic!("balance load failed: {e}");
            #[cfg(not(feature = "dev"))]
            error!("balance load failed: {e}");
        }
    }
}

fn install_balance(commands: &mut Commands, balance: Balance) {
    commands.insert_resource(balance.mobs.clone());
    commands.insert_resource(balance.waves.clone());
    commands.insert_resource(balance.rune_costs.clone());
    commands.insert_resource(balance.globals.clone());
    commands.insert_resource(balance);
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
pub fn reload_balance(input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if !input.just_pressed(KeyCode::F5) {
        return;
    }
    match load_balance() {
        Ok(balance) => {
            install_balance(&mut commands, balance);
            info!("Balance reloaded");
        }
        Err(e) => error!("Balance reload failed: {e}"),
    }
}
