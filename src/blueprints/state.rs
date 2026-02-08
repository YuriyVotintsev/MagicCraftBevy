use bevy::prelude::*;

use crate::stats::ComputedStats;
use super::entity_def::StatesBlock;
use super::recalc::StoredComponentDefs;
use super::SpawnSource;

#[derive(Message)]
pub struct StateTransition {
    pub entity: Entity,
    pub to: usize,
}

#[derive(Component)]
pub struct CurrentState(pub usize);

#[derive(Component)]
pub struct StoredStatesBlock(pub StatesBlock);

pub fn state_transition_system(
    mut commands: Commands,
    mut events: MessageReader<StateTransition>,
    mut query: Query<(
        &mut CurrentState,
        &StoredStatesBlock,
        &SpawnSource,
        &ComputedStats,
        Option<&StoredComponentDefs>,
    )>,
) {
    for event in events.read() {
        let Ok((mut current_state, stored, source, stats, existing_stored_defs)) = query.get_mut(event.entity) else {
            continue;
        };

        let old_idx = current_state.0;
        let new_idx = event.to;

        if old_idx == new_idx {
            continue;
        }

        if let Some(old_state) = stored.0.states.get(old_idx) {
            let mut ec = commands.entity(event.entity);
            for comp in &old_state.components {
                comp.remove_component(&mut ec);
            }
            for trans in &old_state.transitions {
                trans.remove_component(&mut ec);
            }
        }

        let mut new_state_recalc = Vec::new();
        if let Some(new_state) = stored.0.states.get(new_idx) {
            let mut ec = commands.entity(event.entity);
            for comp in &new_state.components {
                comp.insert_component(&mut ec, source, stats);
            }
            for trans in &new_state.transitions {
                trans.insert_component(&mut ec, source, stats);
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

        current_state.0 = new_idx;

        debug!(
            "State transition: '{}' -> '{}'",
            stored.0.state_names[old_idx], stored.0.state_names[new_idx]
        );
    }
}
