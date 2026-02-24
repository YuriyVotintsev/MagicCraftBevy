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
const TEXT_FLOAT_HEIGHT: f32 = 50.0;
const SHADOW_OFFSET: Vec2 = Vec2::new(1.5, -1.5);

#[derive(Component)]
pub struct DamageNumberWorldPos(pub Vec3);

struct ParabolicLens {
    start: Vec3,
    vx: f32,
    vy: f32,
    vz: f32,
}

impl Lens<DamageNumberWorldPos> for ParabolicLens {
    fn lerp(&mut self, mut target: Mut<DamageNumberWorldPos>, ratio: f32) {
        let t = ratio * DURATION;
        target.0.x = self.start.x + self.vx * t;
        target.0.y = self.start.y + self.vy * t - 0.5 * GRAVITY * t * t;
        target.0.z = self.start.z + self.vz * t;
    }
}

pub fn spawn_damage_numbers(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut counter: Local<u64>,
) {
    for event in events.read() {
        *counter += 1;
        let position = Vec3::new(
            event.position.x,
            event.position.y + TEXT_FLOAT_HEIGHT,
            event.position.z,
        );

        let color = match (event.target_faction, event.is_crit) {
            (Faction::Enemy, true) => Color::srgb(1.0, 0.9, 0.1),
            (Faction::Player, _) => Color::srgb(1.0, 0.3, 0.3),
            (Faction::Enemy, false) => Color::WHITE,
        };

        let font_size = if event.is_crit { FONT_SIZE * 1.5 } else { FONT_SIZE };

        let vx = (rand::random::<f32>() - 0.5) * 2.0 * HORIZONTAL_SPEED;
        let vy =
            MIN_VERTICAL_SPEED + rand::random::<f32>() * (MAX_VERTICAL_SPEED - MIN_VERTICAL_SPEED);
        let vz = (rand::random::<f32>() - 0.5) * 2.0 * HORIZONTAL_SPEED;

        let parabola_tween = Tween::new(
            EaseFunction::Linear,
            Duration::from_secs_f32(DURATION),
            ParabolicLens {
                start: position,
                vx,
                vy,
                vz,
            },
        );

        commands.spawn((
            Name::new(format!("DamageNum_{}", *counter)),
            DamageNumber {
                timer: Timer::from_seconds(DURATION, TimerMode::Once),
                color,
            },
            DamageNumberWorldPos(position),
            Text2d::new(format!("{}", event.amount as i32)),
            TextFont {
                font_size,
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

pub fn project_damage_numbers(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut numbers: Query<(&DamageNumberWorldPos, &mut Transform), With<DamageNumber>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    for (world_pos, mut transform) in &mut numbers {
        if let Ok(viewport) = camera.world_to_viewport(cam_gt, world_pos.0) {
            transform.translation.x = viewport.x - half_w;
            transform.translation.y = half_h - viewport.y;
            transform.translation.z = 0.0;
        }
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
