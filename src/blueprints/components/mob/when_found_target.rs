use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::blueprints::context::TargetInfo;
use crate::blueprints::components::ability::find_nearest_enemy::FoundTarget;

#[blueprint_component]
pub struct WhenFoundTarget {
    pub to: StateRef,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        when_found_target_system.in_set(crate::schedule::GameSet::MobAI),
    );
}

fn when_found_target_system(
    mut commands: Commands,
    mut query: Query<(Entity, &WhenFoundTarget, &FoundTarget, &mut SpawnSource)>,
    mut events: MessageWriter<crate::blueprints::state::StateTransition>,
) {
    for (entity, when_found, found_target, mut source) in &mut query {
        source.target = TargetInfo::from_entity_and_position(found_target.0, found_target.1.truncate());
        commands.entity(entity).remove::<FoundTarget>();
        events.write(crate::blueprints::state::StateTransition {
            entity,
            to: when_found.to,
        });
    }
}
