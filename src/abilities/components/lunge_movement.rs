use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;

#[ability_component]
pub struct LungeMovement {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
    #[default_expr("stat(movement_speed)")]
    pub speed: ScalarExpr,
    #[raw(default = 0.6)]
    pub lunge_duration: ScalarExpr,
    #[raw(default = 0.4)]
    pub pause_duration: ScalarExpr,
}

#[derive(Clone, Copy, PartialEq)]
enum LungePhase {
    Lunging,
    Pausing,
}

#[derive(Component)]
pub struct LungeMovementState {
    phase: LungePhase,
    elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_lunge_movement, lunge_movement_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
}

fn init_lunge_movement(
    mut commands: Commands,
    query: Query<Entity, Added<LungeMovement>>,
) {
    for entity in &query {
        commands.entity(entity).insert(LungeMovementState {
            phase: LungePhase::Lunging,
            elapsed: 0.0,
        });
    }
}

fn lunge_movement_system(
    time: Res<Time>,
    mut query: Query<(
        &Transform,
        &mut LinearVelocity,
        &LungeMovement,
        &mut LungeMovementState,
    )>,
    transforms: Query<&Transform, Without<LungeMovement>>,
) {
    let dt = time.delta_secs();

    for (transform, mut velocity, lunge, mut state) in &mut query {
        state.elapsed += dt;

        match state.phase {
            LungePhase::Lunging => {
                if state.elapsed >= lunge.lunge_duration {
                    state.phase = LungePhase::Pausing;
                    state.elapsed = 0.0;
                    velocity.0 = Vec2::ZERO;
                    continue;
                }

                let Ok(target_transform) = transforms.get(lunge.target) else {
                    velocity.0 = Vec2::ZERO;
                    continue;
                };

                let direction = (target_transform.translation - transform.translation).truncate();

                if direction.length_squared() > 1.0 {
                    let t = state.elapsed / lunge.lunge_duration;
                    let factor = (std::f32::consts::PI * t).sin();
                    velocity.0 = direction.normalize() * lunge.speed * factor;
                } else {
                    velocity.0 = Vec2::ZERO;
                }
            }
            LungePhase::Pausing => {
                velocity.0 = Vec2::ZERO;
                if state.elapsed >= lunge.pause_duration {
                    state.phase = LungePhase::Lunging;
                    state.elapsed = 0.0;
                }
            }
        }
    }
}
