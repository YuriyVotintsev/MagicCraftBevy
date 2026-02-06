use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

#[ability_component]
pub struct Orbiting {
    pub radius: ScalarExpr,
    pub angular_speed: ScalarExpr,
    #[default_expr("6.28318 * index / count")]
    pub current_angle: ScalarExpr,
    #[default_expr("caster.entity")]
    pub center: EntityExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_orbiting, update_orbiting_positions)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_orbiting(
    mut commands: Commands,
    query: Query<(Entity, &Orbiting), Added<Orbiting>>,
    transforms: Query<&Transform>,
) {
    for (entity, orbiting) in &query {
        let owner_pos = transforms.get(orbiting.center).map(|t| t.translation).unwrap_or_default();
        let offset = Vec2::new(orbiting.current_angle.cos(), orbiting.current_angle.sin()) * orbiting.radius;
        let position = owner_pos + offset.extend(0.0);
        commands.entity(entity).insert((
            AttachedTo { owner: orbiting.center },
            Transform::from_translation(position),
        ));
    }
}

fn update_orbiting_positions(
    time: Res<Time>,
    owner_query: Query<&Transform, Without<Orbiting>>,
    mut orb_query: Query<(&AttachedTo, &mut Orbiting, &mut Transform)>,
) {
    for (attached, mut orbiting, mut transform) in &mut orb_query {
        orbiting.current_angle += orbiting.angular_speed * time.delta_secs();

        if let Ok(owner_transform) = owner_query.get(attached.owner) {
            let offset = Vec2::new(
                orbiting.current_angle.cos() * orbiting.radius,
                orbiting.current_angle.sin() * orbiting.radius,
            );
            transform.translation = owner_transform.translation + offset.extend(0.0);
        }
    }
}
