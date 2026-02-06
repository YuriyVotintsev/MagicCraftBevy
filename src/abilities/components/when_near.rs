use bevy::prelude::*;
use magic_craft_macros::ability_component;

#[ability_component]
pub struct WhenNear {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
    pub to: String,
    pub distance: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        when_near_system.in_set(crate::schedule::GameSet::MobAI),
    );
}

fn when_near_system(
    query: Query<(Entity, &WhenNear, &Transform)>,
    transforms: Query<&Transform, Without<WhenNear>>,
    mut events: MessageWriter<crate::abilities::state::StateTransition>,
) {
    for (entity, when_near, transform) in &query {
        let Ok(target_transform) = transforms.get(when_near.target) else {
            continue;
        };
        let dist = transform.translation.distance(target_transform.translation);
        if dist < when_near.distance {
            events.write(crate::abilities::state::StateTransition {
                entity,
                to: when_near.to.clone(),
            });
        }
    }
}
