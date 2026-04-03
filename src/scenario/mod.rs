mod runner;
mod types;

use bevy::prelude::*;

pub struct ScenarioPlugin;

impl Plugin for ScenarioPlugin {
    fn build(&self, app: &mut App) {
        let path =
            std::env::var("SCENARIO").expect("ScenarioPlugin requires SCENARIO env var");

        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read scenario '{}': {}", path, e));

        let scenario: types::ScenarioDef = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse scenario '{}': {}", path, e));

        info!("[SCENARIO] Loaded '{}' ({} steps)", path, scenario.steps.len());

        app.insert_resource(runner::ScenarioState::new(scenario))
            .add_systems(
                Update,
                (
                    runner::auto_skip_menu.run_if(in_state(crate::GameState::MainMenu)),
                    runner::scenario_system.before(crate::schedule::GameSet::Input),
                ),
            );
    }
}
