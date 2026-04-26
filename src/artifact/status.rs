use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::components::combat::PendingDamage;
use crate::schedule::GameSet;
use crate::wave::CombatPhase;

#[derive(Component)]
pub struct Burning {
    pub dps: f32,
    pub remaining: f32,
    pub source: Option<Entity>,
}

#[derive(Component)]
pub struct Frozen {
    pub remaining: f32,
    pub slow_pct: f32,
}

pub fn register(app: &mut App) {
    app.add_systems(
        Update,
        (tick_burn, tick_freeze, frozen_movement_modifier)
            .in_set(GameSet::WaveManagement)
            .run_if(in_state(CombatPhase::Running)),
    );
}

fn frozen_movement_modifier(mut q: Query<(&Frozen, &mut LinearVelocity)>) {
    for (frozen, mut velocity) in &mut q {
        let scale = (1.0 - frozen.slow_pct).max(0.0);
        velocity.0 *= scale;
    }
}

fn tick_burn(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Burning)>,
    mut dmg: MessageWriter<PendingDamage>,
) {
    let dt = time.delta_secs();
    for (entity, mut burning) in &mut q {
        let tick_amount = burning.dps * dt;
        if tick_amount > 0.0 {
            dmg.write(PendingDamage {
                target: entity,
                amount: tick_amount,
                source: burning.source,
                on_hit: Default::default(),
            });
        }
        burning.remaining -= dt;
        if burning.remaining <= 0.0 {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.remove::<Burning>();
            }
        }
    }
}

fn tick_freeze(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Frozen)>,
) {
    let dt = time.delta_secs();
    for (entity, mut frozen) in &mut q {
        frozen.remaining -= dt;
        if frozen.remaining <= 0.0 {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.remove::<Frozen>();
            }
        }
    }
}
