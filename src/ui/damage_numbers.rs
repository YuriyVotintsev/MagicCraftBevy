use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use bevy_tweening::{Lens, Tween, TweenAnim};

use crate::Faction;
use crate::stats::DamageEvent;

#[derive(Component)]
pub struct DamageNumber {
    timer: Timer,
    color: Color,
}

const DURATION: f32 = 0.6;
const FONT_SIZE: f32 = 36.0;
const HORIZONTAL_SPEED: f32 = 40.0;
const MIN_VERTICAL_SPEED: f32 = 80.0;
const MAX_VERTICAL_SPEED: f32 = 150.0;
const GRAVITY: f32 = 300.0;
const Z_LAYER: f32 = 100.0;
const SHADOW_OFFSET: Vec2 = Vec2::new(1.5, -1.5);

struct ParabolicLens {
    start: Vec3,
    vx: f32,
    vy: f32,
}

impl Lens<Transform> for ParabolicLens {
    fn lerp(&mut self, mut target: Mut<Transform>, ratio: f32) {
        let t = ratio * DURATION;
        target.translation.x = self.start.x + self.vx * t;
        target.translation.y = self.start.y + self.vy * t - 0.5 * GRAVITY * t * t;
    }
}

pub fn spawn_damage_numbers(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut counter: Local<u64>,
) {
    for event in events.read() {
        *counter += 1;
        let position = event.position + Vec3::new(0.0, 20.0, Z_LAYER);

        let color = match event.target_faction {
            Faction::Player => Color::srgb(1.0, 0.3, 0.3),
            Faction::Enemy => Color::WHITE,
        };

        let vx = (rand::random::<f32>() - 0.5) * 2.0 * HORIZONTAL_SPEED;
        let vy =
            MIN_VERTICAL_SPEED + rand::random::<f32>() * (MAX_VERTICAL_SPEED - MIN_VERTICAL_SPEED);

        let parabola_tween = Tween::new(
            EaseFunction::Linear,
            Duration::from_secs_f32(DURATION),
            ParabolicLens {
                start: position,
                vx,
                vy,
            },
        );

        commands.spawn((
            Name::new(format!("DamageNum_{}", *counter)),
            DamageNumber {
                timer: Timer::from_seconds(DURATION, TimerMode::Once),
                color,
            },
            Text2d::new(format!("{}", event.amount as i32)),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(color),
            Text2dShadow {
                offset: SHADOW_OFFSET,
                color: Color::BLACK,
            },
            Transform::from_translation(position),
            TweenAnim::new(parabola_tween),
        ));
    }
}

pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamageNumber, &mut TextColor, &mut Text2dShadow)>,
) {
    for (entity, mut dn, mut text_color, mut shadow) in &mut query {
        dn.timer.tick(time.delta());
        let t = dn.timer.fraction();
        let alpha = 1.0 - t * t;
        text_color.0 = dn.color.with_alpha(alpha);
        shadow.color = Color::BLACK.with_alpha(alpha);
        if dn.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
