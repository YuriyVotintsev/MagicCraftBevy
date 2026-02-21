use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::sprite::Sprite as SpriteComp;
use crate::stats::{ComputedStats, StatRegistry};

#[blueprint_component]
pub struct SquishWalkAnimation {
    #[raw(default = 0.35)]
    pub amount: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, animate);
}

pub fn animate(
    stat_registry: Option<Res<StatRegistry>>,
    mut query: Query<(&SquishWalkAnimation, &SpriteComp, &mut Transform, &ChildOf)>,
    parent_query: Query<(&LinearVelocity, &ComputedStats)>,
) {
    let Some(stat_registry) = stat_registry else {
        return;
    };
    let speed_id = stat_registry.get("movement_speed");
    for (anim, sprite, mut transform, child_of) in &mut query {
        let t = parent_query
            .get(child_of.parent())
            .ok()
            .and_then(|(vel, stats)| {
                let max = speed_id.map(|id| stats.get(id)).unwrap_or_default();
                if max > 0.0 {
                    Some((vel.length() / max).clamp(0.0, 1.0))
                } else {
                    None
                }
            })
            .unwrap_or(0.0);

        let s = anim.amount * t;
        transform.scale = Vec3::new(1.0 + s, 1.0 - s, 1.0);
        transform.translation.y = sprite.position.y - s * sprite.scale / 2.0;
    }
}
