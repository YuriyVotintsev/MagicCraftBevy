use std::collections::HashSet;

use bevy::prelude::*;

use crate::blueprints::components::common::health::Health;
use crate::player::Player;
use crate::Faction;

use super::types::*;

#[derive(Resource)]
pub struct ScenarioState {
    steps: Vec<Step>,
    current_step: usize,
    elapsed: f32,
    held_keys: HashSet<KeyCode>,
    started: bool,
    assertions_passed: usize,
    assertions_failed: usize,
    initial_time_scale: f32,
    timeout: f32,
}

impl ScenarioState {
    pub fn new(scenario: ScenarioDef) -> Self {
        Self {
            initial_time_scale: scenario.time_scale,
            timeout: scenario.timeout,
            steps: scenario.steps,
            current_step: 0,
            elapsed: 0.0,
            held_keys: HashSet::new(),
            started: false,
            assertions_passed: 0,
            assertions_failed: 0,
        }
    }

    fn total_assertions(&self) -> usize {
        self.assertions_passed + self.assertions_failed
    }
}

pub fn auto_skip_menu(mut next_state: ResMut<NextState<crate::GameState>>) {
    next_state.set(crate::GameState::Playing);
}

pub fn scenario_system(
    time: Res<Time>,
    game_state: Res<State<crate::GameState>>,
    mut state: ResMut<ScenarioState>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
    player_query: Query<(&Transform, Option<&Health>), With<Player>>,
    mob_query: Query<&Faction>,
) {
    if !state.started {
        if *game_state.get() == crate::GameState::Playing {
            state.started = true;
            virtual_time.set_relative_speed(state.initial_time_scale);
            info!(
                "[SCENARIO] Started (time_scale: {}, timeout: {}s, steps: {})",
                state.initial_time_scale,
                state.timeout,
                state.steps.len()
            );
        }
        return;
    }

    state.elapsed += time.delta_secs();

    if state.elapsed >= state.timeout {
        warn!(
            "[SCENARIO] TIMEOUT at {:.1}s (limit: {}s)",
            state.elapsed, state.timeout
        );
        finish(&state, &mut exit);
        return;
    }

    while state.current_step < state.steps.len() {
        if state.steps[state.current_step].at > state.elapsed {
            break;
        }

        let step = state.steps[state.current_step].clone();
        let step_num = state.current_step + 1;
        let total = state.steps.len();

        for action in &step.actions {
            execute_action(
                action,
                &mut state,
                &mut keyboard,
                &mut virtual_time,
                &player_query,
                &mob_query,
                step_num,
                total,
                step.at,
            );
        }

        for assertion in &step.assert {
            check_assertion(
                assertion,
                &mut state,
                &player_query,
                &mob_query,
                step_num,
                total,
                step.at,
            );
        }

        state.current_step += 1;
    }

    if state.current_step >= state.steps.len() {
        finish(&state, &mut exit);
    }
}

fn execute_action(
    action: &Action,
    state: &mut ScenarioState,
    keyboard: &mut ButtonInput<KeyCode>,
    virtual_time: &mut Time<Virtual>,
    player_query: &Query<(&Transform, Option<&Health>), With<Player>>,
    mob_query: &Query<&Faction>,
    step_num: usize,
    total: usize,
    at: f32,
) {
    let prefix = format!("[SCENARIO] Step {}/{} @ {:.1}s:", step_num, total, at);

    match action {
        Action::MoveDir(x, y) => {
            release_movement(state, keyboard);
            if *x > 0.0 {
                hold(state, keyboard, KeyCode::KeyD);
            }
            if *x < 0.0 {
                hold(state, keyboard, KeyCode::KeyA);
            }
            if *y > 0.0 {
                hold(state, keyboard, KeyCode::KeyW);
            }
            if *y < 0.0 {
                hold(state, keyboard, KeyCode::KeyS);
            }
            info!("{} MoveDir({}, {})", prefix, x, y);
        }
        Action::StopMove => {
            release_movement(state, keyboard);
            info!("{} StopMove", prefix);
        }
        Action::Press(key_name) => {
            if let Some(key) = key_from_str(key_name) {
                hold(state, keyboard, key);
                info!("{} Press({})", prefix, key_name);
            } else {
                warn!("{} Press({}) - UNKNOWN KEY", prefix, key_name);
            }
        }
        Action::Release(key_name) => {
            if let Some(key) = key_from_str(key_name) {
                unhold(state, keyboard, key);
                info!("{} Release({})", prefix, key_name);
            } else {
                warn!("{} Release({}) - UNKNOWN KEY", prefix, key_name);
            }
        }
        Action::ReleaseAll => {
            for key in state.held_keys.drain() {
                keyboard.release(key);
            }
            info!("{} ReleaseAll", prefix);
        }
        Action::SetTimeScale(scale) => {
            virtual_time.set_relative_speed(*scale);
            info!("{} SetTimeScale({})", prefix, scale);
        }
        Action::DumpState => {
            dump_state(&prefix, player_query, mob_query, state.elapsed);
        }
        Action::Log(msg) => {
            info!("{} {}", prefix, msg);
        }
    }
}

fn check_assertion(
    assertion: &Assertion,
    state: &mut ScenarioState,
    player_query: &Query<(&Transform, Option<&Health>), With<Player>>,
    mob_query: &Query<&Faction>,
    step_num: usize,
    total: usize,
    at: f32,
) {
    let prefix = format!("[SCENARIO] Step {}/{} @ {:.1}s:", step_num, total, at);

    match assertion {
        Assertion::DumpState => {
            dump_state(&prefix, player_query, mob_query, state.elapsed);
            state.assertions_passed += 1;
        }
        Assertion::PlayerAlive => {
            let alive = player_query.iter().next().is_some();
            log_assert(&prefix, "PlayerAlive", alive, &alive.to_string(), state);
        }
        Assertion::PlayerDead => {
            let dead = player_query.iter().next().is_none();
            log_assert(&prefix, "PlayerDead", dead, &dead.to_string(), state);
        }
        Assertion::PlayerHealth(cmp, expected) => {
            if let Some((_, Some(health))) = player_query.iter().next() {
                let passed = cmp.check_f32(health.current, *expected);
                log_assert(
                    &prefix,
                    &format!("PlayerHealth({} {})", cmp.symbol(), expected),
                    passed,
                    &format!("{:.1}", health.current),
                    state,
                );
            } else {
                log_assert(&prefix, "PlayerHealth", false, "no player", state);
            }
        }
        Assertion::PlayerPosX(cmp, expected) => {
            if let Some((transform, _)) = player_query.iter().next() {
                let actual = transform.translation.x;
                let passed = cmp.check_f32(actual, *expected);
                log_assert(
                    &prefix,
                    &format!("PlayerPosX({} {})", cmp.symbol(), expected),
                    passed,
                    &format!("{:.1}", actual),
                    state,
                );
            } else {
                log_assert(&prefix, "PlayerPosX", false, "no player", state);
            }
        }
        Assertion::PlayerPosY(cmp, expected) => {
            if let Some((transform, _)) = player_query.iter().next() {
                let actual = -transform.translation.z;
                let passed = cmp.check_f32(actual, *expected);
                log_assert(
                    &prefix,
                    &format!("PlayerPosY({} {})", cmp.symbol(), expected),
                    passed,
                    &format!("{:.1}", actual),
                    state,
                );
            } else {
                log_assert(&prefix, "PlayerPosY", false, "no player", state);
            }
        }
        Assertion::MobCount(cmp, expected) => {
            let count = mob_query
                .iter()
                .filter(|f| **f == Faction::Enemy)
                .count();
            let passed = cmp.check_usize(count, *expected);
            log_assert(
                &prefix,
                &format!("MobCount({} {})", cmp.symbol(), expected),
                passed,
                &count.to_string(),
                state,
            );
        }
    }
}

fn log_assert(
    prefix: &str,
    name: &str,
    passed: bool,
    actual: &str,
    state: &mut ScenarioState,
) {
    let tag = if passed { "PASS" } else { "FAIL" };
    if passed {
        info!("{} ASSERT {} => {} {}", prefix, name, actual, tag);
        state.assertions_passed += 1;
    } else {
        warn!("{} ASSERT {} => {} {}", prefix, name, actual, tag);
        state.assertions_failed += 1;
    }
}

fn dump_state(
    prefix: &str,
    player_query: &Query<(&Transform, Option<&Health>), With<Player>>,
    mob_query: &Query<&Faction>,
    elapsed: f32,
) {
    info!("{} === State Dump (game time: {:.1}s) ===", prefix, elapsed);

    if let Some((transform, health)) = player_query.iter().next() {
        let pos = transform.translation;
        let hp = health.map(|h| format!("{:.1}", h.current)).unwrap_or("?".into());
        info!(
            "{}   Player: pos=({:.0}, {:.0}) hp={}",
            prefix, pos.x, -pos.z, hp
        );
    } else {
        info!("{}   Player: NONE", prefix);
    }

    let mob_count = mob_query
        .iter()
        .filter(|f| **f == Faction::Enemy)
        .count();
    info!("{}   Mobs: {}", prefix, mob_count);
}

fn finish(state: &ScenarioState, exit: &mut MessageWriter<AppExit>) {
    info!("[SCENARIO] ===================================");
    if state.total_assertions() == 0 {
        info!("[SCENARIO] DONE (no assertions)");
    } else if state.assertions_failed == 0 {
        info!(
            "[SCENARIO] PASS ({}/{} assertions passed)",
            state.assertions_passed,
            state.total_assertions()
        );
    } else {
        warn!(
            "[SCENARIO] FAIL ({} passed, {} failed out of {})",
            state.assertions_passed,
            state.assertions_failed,
            state.total_assertions()
        );
    }
    info!("[SCENARIO] ===================================");

    let code = if state.assertions_failed == 0 {
        AppExit::Success
    } else {
        AppExit::error()
    };
    exit.write(code);
}

fn hold(state: &mut ScenarioState, keyboard: &mut ButtonInput<KeyCode>, key: KeyCode) {
    state.held_keys.insert(key);
    keyboard.press(key);
}

fn unhold(state: &mut ScenarioState, keyboard: &mut ButtonInput<KeyCode>, key: KeyCode) {
    state.held_keys.remove(&key);
    keyboard.release(key);
}

fn release_movement(state: &mut ScenarioState, keyboard: &mut ButtonInput<KeyCode>) {
    for key in [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD] {
        if state.held_keys.remove(&key) {
            keyboard.release(key);
        }
    }
}

fn key_from_str(s: &str) -> Option<KeyCode> {
    match s {
        "W" => Some(KeyCode::KeyW),
        "A" => Some(KeyCode::KeyA),
        "S" => Some(KeyCode::KeyS),
        "D" => Some(KeyCode::KeyD),
        "Q" => Some(KeyCode::KeyQ),
        "E" => Some(KeyCode::KeyE),
        "R" => Some(KeyCode::KeyR),
        "F" => Some(KeyCode::KeyF),
        "Space" => Some(KeyCode::Space),
        "Escape" => Some(KeyCode::Escape),
        "Enter" => Some(KeyCode::Enter),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "Tab" => Some(KeyCode::Tab),
        "1" => Some(KeyCode::Digit1),
        "2" => Some(KeyCode::Digit2),
        "3" => Some(KeyCode::Digit3),
        "4" => Some(KeyCode::Digit4),
        "5" => Some(KeyCode::Digit5),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "Backquote" => Some(KeyCode::Backquote),
        _ => None,
    }
}
