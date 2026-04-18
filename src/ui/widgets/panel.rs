use bevy::prelude::*;

use crate::palette;

pub const PANEL_RADIUS: f32 = 20.0;
pub const PANEL_BORDER_WIDTH: f32 = 3.0;
pub const PANEL_SHADOW_OFFSET: f32 = 6.0;

pub fn panel_node(mut node: Node, border: Option<Color>) -> impl Bundle {
    node.border = UiRect::all(Val::Px(PANEL_BORDER_WIDTH));
    node.border_radius = BorderRadius::all(Val::Px(PANEL_RADIUS));
    (
        node,
        BackgroundColor(palette::color("ui_panel_bg")),
        BorderColor::all(border.unwrap_or_else(|| palette::color("ui_panel_border"))),
        BoxShadow::new(
            palette::color("ui_panel_shadow"),
            Val::ZERO,
            Val::Px(PANEL_SHADOW_OFFSET),
            Val::ZERO,
            Val::ZERO,
        ),
    )
}
