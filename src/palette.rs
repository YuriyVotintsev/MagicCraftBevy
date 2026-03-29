use std::collections::HashMap;
use std::sync::OnceLock;

use bevy::prelude::Color;
use serde::Deserialize;

static PALETTE: OnceLock<PaletteData> = OnceLock::new();

#[derive(Deserialize)]
struct PaletteData {
    colors: HashMap<String, (f32, f32, f32)>,
    aliases: HashMap<String, String>,
}

impl PaletteData {
    fn lookup(&self, name: &str) -> Option<(f32, f32, f32)> {
        let color_name = self.aliases.get(name).map(|s| s.as_str()).unwrap_or(name);
        self.colors.get(color_name).copied()
    }
}

pub fn init() {
    let content = std::fs::read_to_string("assets/palette.ron")
        .expect("Failed to read assets/palette.ron");
    let data: PaletteData =
        ron::from_str(&content).expect("Failed to parse assets/palette.ron");
    PALETTE.set(data).ok();
}

pub fn lookup(name: &str) -> Option<(f32, f32, f32)> {
    PALETTE.get().and_then(|p| p.lookup(name))
}

pub fn color(name: &str) -> Color {
    let (r, g, b) = lookup(name).unwrap_or_else(|| panic!("Unknown palette color: {name}"));
    Color::srgb(r, g, b)
}

pub fn color_alpha(name: &str, alpha: f32) -> Color {
    let (r, g, b) = lookup(name).unwrap_or_else(|| panic!("Unknown palette color: {name}"));
    Color::srgba(r, g, b, alpha)
}
