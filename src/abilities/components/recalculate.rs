use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::spawn::{SpawnContext, StoredComponentDefs};
use crate::abilities::AbilitySource;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[ability_component]
pub struct Recalculate {
    #[raw(default = 0)]
    pub interval: ScalarExpr,
}

#[derive(Component, Default)]
pub struct RecalculateTimer {
    pub timer: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_recalculate_timer, recalculate_system)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_recalculate_timer(
    mut commands: Commands,
    query: Query<Entity, Added<Recalculate>>,
) {
    for entity in &query {
        commands.entity(entity).insert(RecalculateTimer::default());
    }
}

fn recalculate_system(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &AbilitySource,
        &Recalculate,
        &mut RecalculateTimer,
        &StoredComponentDefs,
    )>,
    stats_query: Query<&ComputedStats, Changed<ComputedStats>>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (entity, source, recalculate, mut timer, stored) in &mut query {
        let caster_entity = source.caster.entity.unwrap();
        if stats_query.get(caster_entity).is_err() {
            continue;
        }

        if recalculate.interval > 0.0 {
            timer.timer -= delta;
            if timer.timer > 0.0 {
                continue;
            }
            timer.timer = recalculate.interval;
        }

        let defs = stored.defs.clone();
        let ability_id = source.ability_id;
        let caster_entity = caster_entity;
        let caster_faction = source.caster_faction;

        commands.queue(RecalculateCommand {
            entity,
            defs,
            ability_id,
            caster: caster_entity,
            caster_faction,
        });
    }
}

struct RecalculateCommand {
    entity: Entity,
    defs: Vec<crate::abilities::components::ComponentDef>,
    ability_id: crate::abilities::ids::AbilityId,
    caster: Entity,
    caster_faction: crate::Faction,
}

impl Command for RecalculateCommand {
    fn apply(self, world: &mut World) {
        let caster_pos = world
            .get::<Transform>(self.caster)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let caster_stats = world
            .get::<ComputedStats>(self.caster)
            .cloned()
            .unwrap_or_else(|| DEFAULT_STATS.clone());

        let caster_info = TargetInfo::from_entity_and_position(self.caster, caster_pos);

        let ctx = SpawnContext {
            ability_id: self.ability_id,
            caster: caster_info,
            caster_faction: self.caster_faction,
            source: caster_info,
            target: TargetInfo::EMPTY,
            stats: &caster_stats,
            index: 0,
            count: 1,
        };

        for def in &self.defs {
            def.update_component(self.entity, &ctx, world);
        }
    }
}
