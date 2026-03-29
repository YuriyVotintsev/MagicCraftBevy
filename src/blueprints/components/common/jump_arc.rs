use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::mob::jump_toward::{JumpPhase, JumpToward, JumpTowardState};

#[blueprint_component]
pub struct JumpArc {
    #[raw(default = 0.5)]
    pub arc_height: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, animate_jump_arc);
}

fn animate_jump_arc(
    mut query: Query<(&JumpArc, &mut Transform, &ChildOf)>,
    parent_query: Query<(&JumpTowardState, &JumpToward)>,
) {
    for (arc, mut transform, child_of) in &mut query {
        let Ok((state, jump)) = parent_query.get(child_of.parent()) else {
            transform.translation.y = 0.0;
            transform.rotation = Quat::IDENTITY;
            transform.scale = Vec3::ONE;
            continue;
        };

        match state.phase {
            JumpPhase::Charging => {
                let t = (state.elapsed / jump.charge_duration).min(1.0);
                let squash_t = t * t;
                let scale_y = 1.0 - 0.3 * squash_t;
                let scale_x = 1.0 / scale_y.sqrt();

                transform.translation.y = -0.5 * (1.0 - scale_y);
                transform.rotation = Quat::IDENTITY;
                transform.scale = Vec3::new(scale_x, scale_y, 1.0);
            }
            JumpPhase::Flying => {
                let t = (state.elapsed / jump.flight_duration).min(1.0);
                let h = (std::f32::consts::PI * t).sin();

                transform.translation.y = h * arc.arc_height;

                let angle = state.direction.y.atan2(state.direction.x);
                transform.rotation = Quat::from_rotation_z(angle);

                let ng = 1.0 - h;
                let squash = ng.powi(3) * 0.3;
                let stretch = h * ng.powi(2) * 1.2;
                let ss = stretch - squash;
                let scale_along = 1.0 + ss;
                let scale_perp = 1.0 / scale_along.sqrt();

                transform.scale = Vec3::new(scale_along, scale_perp, 1.0);
            }
            JumpPhase::Landing => {
                let t = (state.elapsed / jump.land_duration).min(1.0);
                let squash_t = (1.0 - t).powi(3);
                let scale_y = 1.0 - 0.35 * squash_t;
                let scale_x = 1.0 / scale_y.sqrt();

                transform.translation.y = -0.5 * (1.0 - scale_y);
                transform.rotation = Quat::IDENTITY;
                transform.scale = Vec3::new(scale_x, scale_y, 1.0);
            }
        }
    }
}
