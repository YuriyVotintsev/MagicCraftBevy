use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_activator;
use crate::abilities::expr::ScalarExpr;
use crate::abilities::eval_context::EvalContext;
use crate::abilities::{ActivateAbilityEvent, AbilityContext, TargetInfo, ProvidedFields, AbilityInstance};
use crate::stats::ComputedStats;
use crate::schedule::GameSet;
use crate::{Faction, GameState};

#[derive(Debug, Clone, Default, GenerateRaw)]
#[activator]
pub struct IntervalParams {
    pub interval: ScalarExpr,
    #[raw(default = false)]
    pub skip_first: bool,
}

#[derive(Component)]
pub struct IntervalActivator {
    pub interval: ScalarExpr,
    pub timer: f32,
    pub skip_first: bool,
    pub activated: bool,
}

impl IntervalActivator {
    pub fn from_params(params: &IntervalParams) -> Self {
        Self {
            interval: params.interval.clone(),
            timer: 0.0,
            skip_first: params.skip_first,
            activated: false,
        }
    }
}

pub fn provided_fields() -> ProvidedFields {
    ProvidedFields::SOURCE_ENTITY
        .union(ProvidedFields::SOURCE_POSITION)
}

fn interval_system(
    time: Res<Time>,
    mut trigger_events: MessageWriter<ActivateAbilityEvent>,
    mut ability_query: Query<(&AbilityInstance, &mut IntervalActivator)>,
    owner_query: Query<(&Transform, &Faction, &ComputedStats)>,
) {
    let delta = time.delta_secs();

    for (instance, mut activator) in &mut ability_query {
        if activator.skip_first && !activator.activated {
            let Ok((_, _, stats)) = owner_query.get(instance.owner) else { continue };
            activator.timer = activator.interval.eval(&EvalContext::stats_only(stats));
            activator.activated = true;
            continue;
        }

        activator.timer -= delta;
        if activator.timer > 0.0 { continue }

        let Ok((transform, faction, stats)) = owner_query.get(instance.owner) else { continue };

        let source = TargetInfo::from_entity_and_position(instance.owner, transform.translation.truncate());
        let target = TargetInfo::EMPTY;

        let ctx = AbilityContext::new(
            instance.owner,
            *faction,
            source,
            target,
        );

        trigger_events.write(ActivateAbilityEvent {
            ability_id: instance.ability_id,
            context: ctx,
        });

        activator.timer = activator.interval.eval(&EvalContext::stats_only(stats));
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        interval_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

register_activator!(IntervalParams, IntervalActivator);
