use bevy::prelude::*;

use crate::blueprints::components::common::jump_walk_animation::animate as jump_animate;
use crate::blueprints::components::common::squish_walk_animation::animate as squish_animate;

#[derive(Component)]
pub struct HitFlash {
    elapsed: f32,
    duration: f32,
}

impl HitFlash {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            duration: 0.3,
        }
    }
}

pub struct HitFlashPlugin;

impl Plugin for HitFlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            tick_hit_flash
                .after(jump_animate)
                .after(squish_animate),
        );
    }
}

fn tick_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut HitFlash, &Children)>,
    mut sprite_query: Query<(&mut Sprite, &mut Transform)>,
) {
    for (entity, mut flash, children) in &mut flash_query {
        flash.elapsed += time.delta_secs();
        let t = (flash.elapsed / flash.duration).clamp(0.0, 1.0);

        let done = t >= 1.0;

        for child in children.iter() {
            if let Ok((mut sprite, mut transform)) = sprite_query.get_mut(child) {
                if done {
                    sprite.color = Color::WHITE;
                } else {
                    let brightness = 6.0_f32.lerp(1.0, t);
                    let scale_x = 0.7_f32.lerp(1.0, t);
                    let scale_y = 1.3_f32.lerp(1.0, t);
                    sprite.color = Color::srgb(brightness, brightness, brightness);
                    transform.scale.x *= scale_x;
                    transform.scale.y *= scale_y;
                }
            }
        }

        if done {
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}
