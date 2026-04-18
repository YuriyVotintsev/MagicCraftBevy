use bevy::prelude::*;

use crate::palette;

pub const BUTTON_RADIUS: f32 = 200.0;
pub const BUTTON_BORDER_WIDTH: f32 = 3.0;
pub const BUTTON_SHADOW_OFFSET: f32 = 6.0;

pub fn button_node(mut node: Node, border: Option<Color>) -> impl Bundle {
    node.border = UiRect::all(Val::Px(BUTTON_BORDER_WIDTH));
    node.border_radius = BorderRadius::all(Val::Px(BUTTON_RADIUS));
    (
        Button,
        node,
        BackgroundColor(palette::color("ui_panel_bg")),
        BorderColor::all(border.unwrap_or_else(|| palette::color("ui_panel_border"))),
        BoxShadow::new(
            palette::color("ui_panel_shadow"),
            Val::ZERO,
            Val::Px(BUTTON_SHADOW_OFFSET),
            Val::ZERO,
            Val::ZERO,
        ),
    )
}
