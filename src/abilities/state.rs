use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Faction;
use crate::stats::ComputedStats;
use super::context::TargetInfo;
use super::entity_def::StatesBlock;
use super::spawn::StoredComponentDefs;
use super::AbilitySource;

#[derive(Message)]
pub struct StateTransition {
    pub entity: Entity,
    pub to: String,
}

#[derive(Component)]
pub struct CurrentState(pub String);

#[derive(Component)]
pub struct StoredStatesBlock(pub StatesBlock);

pub fn state_transition_system(
    mut commands: Commands,
    mut events: MessageReader<StateTransition>,
    mut query: Query<(
        &mut CurrentState,
        &StoredStatesBlock,
        &AbilitySource,
        &Faction,
        &Transform,
        &ComputedStats,
        Option<&StoredComponentDefs>,
    )>,
) {
    for event in events.read() {
        let Ok((mut current_state, stored, source, faction, transform, stats, existing_stored_defs)) = query.get_mut(event.entity) else {
            continue;
        };

        let old_state_name = current_state.0.clone();
        let new_state_name = &event.to;

        if old_state_name == *new_state_name {
            continue;
        }

        if let Some(old_state) = stored.0.states.get(&old_state_name) {
            let mut ec = commands.entity(event.entity);
            for comp in &old_state.components {
                comp.remove_component(&mut ec);
            }
            for trans in &old_state.transitions {
                trans.remove_component(&mut ec);
            }
        }

        commands.entity(event.entity).insert(LinearVelocity::ZERO);

        let mut new_state_recalc = Vec::new();
        if let Some(new_state) = stored.0.states.get(new_state_name) {
            let mut caster = source.caster;
            caster.position = Some(transform.translation.truncate());

            let transition_source = AbilitySource {
                ability_id: source.ability_id,
                caster,
                caster_faction: *faction,
                source: TargetInfo::from_entity_and_position(event.entity, transform.translation.truncate()),
                target: TargetInfo::EMPTY,
                index: 0,
                count: 1,
            };

            let mut ec = commands.entity(event.entity);
            for comp in &new_state.components {
                comp.insert_component(&mut ec, &transition_source, stats);
            }
            for trans in &new_state.transitions {
                trans.insert_component(&mut ec, &transition_source, stats);
            }

            new_state_recalc.extend(
                new_state.components.iter()
                    .chain(new_state.transitions.iter())
                    .filter(|c| c.has_recalc())
                    .cloned()
            );
        }

        if let Some(existing) = existing_stored_defs {
            let has_base = !existing.base.is_empty();
            if has_base || !new_state_recalc.is_empty() {
                commands.entity(event.entity).insert(StoredComponentDefs {
                    base: existing.base.clone(),
                    state: new_state_recalc,
                });
            } else {
                commands.entity(event.entity).remove::<StoredComponentDefs>();
            }
        } else if !new_state_recalc.is_empty() {
            commands.entity(event.entity).insert(StoredComponentDefs {
                base: Vec::new(),
                state: new_state_recalc,
            });
        }

        current_state.0 = new_state_name.clone();

        info!(
            "State transition: '{}' -> '{}'",
            old_state_name, new_state_name
        );
    }
}
