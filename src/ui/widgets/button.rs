use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::palette;

pub const BUTTON_RADIUS: f32 = 200.0;
pub const BUTTON_BORDER_WIDTH: f32 = 3.0;
pub const BUTTON_SHADOW_OFFSET: f32 = 5.0;
pub const BUTTON_LIFT: f32 = 3.0;

#[derive(Component, Clone, Copy)]
pub struct WidgetButton {
    rest_top: Val,
    last_interaction: Interaction,
}

impl WidgetButton {
    fn lifted(&self, displacement: f32) -> Val {
        match self.rest_top {
            Val::Auto => Val::Px(displacement),
            Val::Px(b) => Val::Px(b + displacement),
            other => other,
        }
    }

    fn just_released(&self, current: Interaction) -> bool {
        // On touch devices there is no hover state: the pointer goes
        // straight from `Pressed` back to `None` on finger release, so we
        // accept that as a click too. On native desktop we keep the stricter
        // check so that sliding the cursor off mid-press still cancels.
        if self.last_interaction != Interaction::Pressed {
            return false;
        }
        #[cfg(target_arch = "wasm32")]
        {
            matches!(current, Interaction::Hovered | Interaction::None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            current == Interaction::Hovered
        }
    }
}

pub fn button_node(mut node: Node, border: Option<Color>) -> impl Bundle {
    node.border = UiRect::all(Val::Px(BUTTON_BORDER_WIDTH));
    node.border_radius = BorderRadius::all(Val::Px(BUTTON_RADIUS));
    let widget = WidgetButton {
        rest_top: node.top,
        last_interaction: Interaction::None,
    };
    (
        Button,
        widget,
        node,
        BackgroundColor(palette::color("ui_button_normal")),
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

#[derive(SystemParam)]
pub struct ReleasedButtons<'w, 's, M: Component> {
    query: Query<
        'w,
        's,
        (&'static Interaction, &'static WidgetButton, &'static M),
        Changed<Interaction>,
    >,
}

impl<M: Component> ReleasedButtons<'_, '_, M> {
    pub fn for_each(&self, mut handler: impl FnMut(&M)) {
        for (interaction, widget, marker) in &self.query {
            if widget.just_released(*interaction) {
                handler(marker);
            }
        }
    }
}

pub fn button_interaction_visuals(
    mut query: Query<
        (&Interaction, &WidgetButton, &mut Node, &mut BoxShadow),
        Changed<Interaction>,
    >,
) {
    for (interaction, widget, mut node, mut shadow) in &mut query {
        let displacement = match interaction {
            Interaction::Pressed => BUTTON_LIFT,
            Interaction::Hovered => -BUTTON_LIFT,
            Interaction::None => 0.0,
        };
        node.top = widget.lifted(displacement);
        if let Some(s) = shadow.0.first_mut() {
            s.y_offset = Val::Px(BUTTON_SHADOW_OFFSET - displacement);
        }
    }
}

pub fn update_button_last_interaction(
    mut query: Query<(&Interaction, &mut WidgetButton), Changed<Interaction>>,
) {
    for (interaction, mut widget) in &mut query {
        widget.last_interaction = *interaction;
    }
}
