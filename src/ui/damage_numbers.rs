use bevy::prelude::*;

use crate::stats::DamageEvent;

#[derive(Component)]
pub struct DamageNumber {
    timer: Timer,
}

const DURATION: f32 = 0.8;
const FONT_SIZE: f32 = 28.0;

pub fn spawn_damage_numbers(mut commands: Commands, mut events: MessageReader<DamageEvent>) {
    for event in events.read() {
        let offset_x = (rand::random::<f32>() - 0.5) * 20.0;
        let position = event.position + Vec3::new(offset_x, 20.0, 10.0);

        commands.spawn((
            DamageNumber {
                timer: Timer::from_seconds(DURATION, TimerMode::Once),
            },
            Text2d::new(format!("{}", event.amount as i32)),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            Transform::from_translation(position),
        ));
    }
}

pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamageNumber)>,
) {
    for (entity, mut dn) in &mut query {
        dn.timer.tick(time.delta());
        if dn.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
