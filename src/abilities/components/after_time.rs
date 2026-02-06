use bevy::prelude::*;
use magic_craft_macros::ability_component;

#[ability_component]
pub struct AfterTime {
    pub to: String,
    pub duration: ScalarExpr,
}

#[derive(Component)]
pub struct AfterTimeTimer {
    pub elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_after_time_timer, after_time_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
}

fn init_after_time_timer(
    mut commands: Commands,
    query: Query<Entity, Added<AfterTime>>,
) {
    for entity in &query {
        commands.entity(entity).insert(AfterTimeTimer { elapsed: 0.0 });
    }
}

fn after_time_system(
    time: Res<Time>,
    mut query: Query<(Entity, &AfterTime, &mut AfterTimeTimer)>,
    mut events: MessageWriter<crate::fsm::StateTransition>,
) {
    for (entity, after_time, mut timer) in &mut query {
        timer.elapsed += time.delta_secs();
        if timer.elapsed >= after_time.duration {
            events.write(crate::fsm::StateTransition {
                entity,
                to: after_time.to.clone(),
            });
        }
    }
}
