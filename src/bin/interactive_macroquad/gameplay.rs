use macroquad::prelude::*;

use crate::constants::{FIXED_STEP_S, HEIGHT_KEY_RATE_MPS, MAX_SIM_TIME_S, VELOCITY_KEY_RATE_MPS};
use crate::controls::FrameActions;
use crate::model::{GamePhase, StepOutcome};
use crate::physics::step_projectile;
use crate::state::AppRuntime;

pub(crate) fn apply_actions(state: &mut AppRuntime, actions: FrameActions) -> bool {
    if actions.launch_pause {
        match state.game.phase {
            GamePhase::Flying => {
                state.game.paused = !state.game.paused;
                state.game.status_line = if state.game.paused {
                    "Paused".to_string()
                } else {
                    "Resumed".to_string()
                };
            }
            _ => state.game.launch(state.config),
        }
    }

    if actions.reset {
        state.game.reset();
    }

    if actions.prev_level && state.current_level_idx > 0 {
        state.current_level_idx -= 1;
        state.load_current_level_defaults();
        state.set_moved_status();
        return true;
    }

    if actions.next_level && state.current_level_idx < state.highest_unlocked_level {
        state.current_level_idx += 1;
        state.load_current_level_defaults();
        state.set_advanced_status();
        return true;
    }

    false
}

pub(crate) fn apply_keyboard_adjustments(state: &mut AppRuntime, frame_dt: f32) {
    if !is_mouse_button_down(MouseButton::Left) {
        if is_key_down(KeyCode::W) {
            state.config.height_m += HEIGHT_KEY_RATE_MPS * frame_dt;
        }
        if is_key_down(KeyCode::S) {
            state.config.height_m -= HEIGHT_KEY_RATE_MPS * frame_dt;
        }
        if is_key_down(KeyCode::D) {
            state.config.speed_mps += VELOCITY_KEY_RATE_MPS * frame_dt;
        }
        if is_key_down(KeyCode::A) {
            state.config.speed_mps -= VELOCITY_KEY_RATE_MPS * frame_dt;
        }
    }
    state.config.height_m = state.config.height_m.clamp(0.0, 400.0);
    state.config.speed_mps = state.config.speed_mps.clamp(5.0, 500.0);
}

pub(crate) fn step_active_flight(state: &mut AppRuntime, frame_dt: f32) {
    if !matches!(state.game.phase, GamePhase::Flying) || state.game.paused {
        return;
    }

    let mut remaining = (frame_dt * state.sim_speed).min(0.10);
    let level_idx = state.current_level_idx;
    let levels_len = state.levels.len();
    let level = &state.levels[level_idx];
    while remaining > 0.0 {
        let dt = remaining.min(FIXED_STEP_S);
        remaining -= dt;

        let mut status_update: Option<(GamePhase, String)> = None;
        if let Some(shot) = state.game.shot.as_mut() {
            let outcome = step_projectile(shot, level, dt);
            state.game.trail.push(shot.position);

            if outcome != StepOutcome::Flying {
                let next_phase = if outcome == StepOutcome::HitTarget {
                    GamePhase::Success
                } else {
                    GamePhase::Failed
                };
                let status = match outcome {
                    StepOutcome::HitTarget => {
                        if state.current_level_idx == state.highest_unlocked_level
                            && state.highest_unlocked_level + 1 < levels_len
                        {
                            state.highest_unlocked_level += 1;
                        }
                        let unlock_note = if state.current_level_idx < state.highest_unlocked_level
                        {
                            " | Next level unlocked (N)"
                        } else {
                            " | Campaign complete"
                        };
                        format!(
                            "Target hit in {:.2}s with {} bounce(s){}",
                            shot.elapsed_s, shot.bounces, unlock_note
                        )
                    }
                    StepOutcome::HitGround => format!(
                        "Missed target: hit ground at x={:.2} m after {} bounce(s)",
                        shot.position.x.max(0.0),
                        shot.bounces
                    ),
                    StepOutcome::HitBarrier => "Missed target: barrier collision".to_string(),
                    StepOutcome::Flying => String::new(),
                };
                status_update = Some((next_phase, status));
            } else if shot.elapsed_s > MAX_SIM_TIME_S {
                status_update = Some((GamePhase::Failed, "Missed target: timed out".to_string()));
            }
        }

        if let Some((phase, status)) = status_update {
            state.game.phase = phase;
            state.game.status_line = status;
            break;
        }
    }
}
