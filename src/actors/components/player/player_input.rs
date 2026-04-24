use bevy::prelude::*;

use super::super::super::player::{fire_fireball, FIREBALL_COOLDOWN};
use crate::input::PlayerIntent;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::wave::WavePhase;
use crate::Faction;

#[derive(Component)]
pub struct PlayerInput;

#[derive(Component, Default)]
pub struct PlayerAbilityCooldowns {
    pub current: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (tick_cooldowns, intent_fire_system)
            .chain()
            .in_set(GameSet::Input)
            .run_if(in_state(WavePhase::Combat)),
    );
}

fn tick_cooldowns(time: Res<Time>, mut q: Query<&mut PlayerAbilityCooldowns>) {
    let dt = time.delta_secs();
    for mut c in &mut q {
        c.current = (c.current - dt).max(0.0);
    }
}

fn intent_fire_system(
    mut commands: Commands,
    intent: Res<PlayerIntent>,
    mut player_query: Query<
        (
            Entity,
            &Transform,
            Option<&ComputedStats>,
            &mut PlayerAbilityCooldowns,
        ),
        With<PlayerInput>,
    >,
) {
    if !intent.fire {
        return;
    }
    if intent.aim_dir == Vec2::ZERO {
        return;
    }
    for (player_entity, player_transform, stats, mut cooldowns) in &mut player_query {
        if cooldowns.current > 0.0 {
            continue;
        }
        let caster_pos = crate::coord::to_2d(player_transform.translation);
        fire_fireball(
            &mut commands,
            player_entity,
            caster_pos,
            Faction::Player,
            intent.aim_dir,
            stats,
        );
        let attack_speed = stats
            .map(|s| s.final_of(Stat::AttackSpeed))
            .unwrap_or(1.0)
            .max(0.01);
        cooldowns.current = FIREBALL_COOLDOWN / attack_speed;
    }
}
