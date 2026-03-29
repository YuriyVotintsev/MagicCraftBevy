use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct JumpToward {
    #[default_expr("stat(movement_speed)")]
    pub speed: ScalarExpr,
    #[raw(default = 0.2)]
    pub charge_duration: ScalarExpr,
    #[raw(default = 0.6)]
    pub flight_duration: ScalarExpr,
    #[raw(default = 0.2)]
    pub land_duration: ScalarExpr,
}

#[derive(Clone, Copy, PartialEq)]
pub enum JumpPhase {
    Charging,
    Flying,
    Landing,
}

#[derive(Component)]
pub struct JumpTowardState {
    pub phase: JumpPhase,
    pub elapsed: f32,
    pub direction: Vec2,
    pub flight_speed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_jump_toward, jump_toward_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
    app.add_observer(on_remove_jump_toward);
}

fn on_remove_jump_toward(
    on: On<Remove, JumpToward>,
    mut commands: Commands,
    mut q: Query<&mut LinearVelocity>,
) {
    let entity = on.event_target();
    if let Ok(mut v) = q.get_mut(entity) {
        v.0 = Vec2::ZERO;
    }
    commands.entity(entity).queue_silenced(|mut entity: EntityWorldMut| {
        entity.remove::<JumpTowardState>();
    });
}

fn init_jump_toward(
    mut commands: Commands,
    query: Query<Entity, Added<JumpToward>>,
) {
    for entity in &query {
        commands.entity(entity).insert(JumpTowardState {
            phase: JumpPhase::Charging,
            elapsed: 0.0,
            direction: Vec2::ZERO,
            flight_speed: 0.0,
        });
    }
}

fn jump_toward_system(
    time: Res<Time>,
    mut query: Query<(
        &Transform,
        &mut LinearVelocity,
        &JumpToward,
        &mut JumpTowardState,
    )>,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    let dt = time.delta_secs();

    for (transform, mut velocity, jump, mut state) in &mut query {
        state.elapsed += dt;

        match state.phase {
            JumpPhase::Charging => {
                velocity.0 = Vec2::ZERO;

                if state.elapsed >= jump.charge_duration {
                    if let Ok(player_transform) = player_query.single() {
                        let to_player =
                            (player_transform.translation - transform.translation).truncate();
                        let distance = to_player.length();
                        state.direction = if distance > 1.0 {
                            to_player / distance
                        } else {
                            Vec2::X
                        };
                        state.flight_speed = (distance * std::f32::consts::PI
                            / (2.0 * jump.flight_duration))
                            .min(jump.speed);
                    }

                    state.phase = JumpPhase::Flying;
                    state.elapsed = 0.0;
                }
            }
            JumpPhase::Flying => {
                if state.elapsed >= jump.flight_duration {
                    state.phase = JumpPhase::Landing;
                    state.elapsed = 0.0;
                    velocity.0 = Vec2::ZERO;
                    continue;
                }

                let t = state.elapsed / jump.flight_duration;
                let speed_factor = (std::f32::consts::PI * t).sin();
                velocity.0 = state.direction * state.flight_speed * speed_factor;
            }
            JumpPhase::Landing => {
                velocity.0 = Vec2::ZERO;
            }
        }
    }
}
