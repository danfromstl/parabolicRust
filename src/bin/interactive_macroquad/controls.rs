use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use crate::model::GamePhase;
use crate::state::AppRuntime;

#[derive(Default, Clone, Copy)]
pub(crate) struct FrameActions {
    pub(crate) launch_pause: bool,
    pub(crate) reset: bool,
    pub(crate) prev_level: bool,
    pub(crate) next_level: bool,
}

impl FrameActions {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            launch_pause: self.launch_pause || other.launch_pause,
            reset: self.reset || other.reset,
            prev_level: self.prev_level || other.prev_level,
            next_level: self.next_level || other.next_level,
        }
    }
}

pub(crate) fn hotkey_actions() -> FrameActions {
    FrameActions {
        launch_pause: is_key_pressed(KeyCode::Space),
        reset: is_key_pressed(KeyCode::R),
        prev_level: is_key_pressed(KeyCode::P),
        next_level: is_key_pressed(KeyCode::N),
    }
}

pub(crate) fn draw_control_panel(state: &mut AppRuntime) -> FrameActions {
    let level = state.current_level();
    let level_code = level.code;
    let level_env = level.environment;

    let mut actions = FrameActions::default();
    widgets::Window::new(hash!(), vec2(18.0, 120.0), vec2(360.0, 300.0))
        .label(&format!("{} Controls", level_code))
        .ui(&mut *root_ui(), |ui| {
            ui.label(None, &format!("Environment: {}", level_env.name));
            ui.label(
                None,
                &format!(
                    "g = {:.2} m/s^2 | wind = {:.2} | drag = {:.2}",
                    level_env.gravity_mps2, level_env.wind_accel_x_mps2, level_env.drag_linear
                ),
            );
            ui.separator();
            ui.slider(
                hash!(),
                "Angle (deg)",
                -89.0..89.0,
                &mut state.config.angle_deg,
            );
            ui.slider(
                hash!(),
                "Velocity (m/s)",
                5.0..500.0,
                &mut state.config.speed_mps,
            );
            ui.slider(
                hash!(),
                "Height (m)",
                0.0..400.0,
                &mut state.config.height_m,
            );
            ui.slider(hash!(), "Simulation Speed", 0.5..5.0, &mut state.sim_speed);
            ui.separator();
            if ui.button(None, "Launch (Space)") {
                actions.launch_pause = true;
            }
            if ui.button(None, "Reset (R)") {
                actions.reset = true;
            }
            if ui.button(None, "Toggle Preview") {
                state.show_preview = !state.show_preview;
            }
            if ui.button(None, "Prev Level (P)") {
                actions.prev_level = true;
            }
            if ui.button(None, "Next Level (N)") {
                actions.next_level = true;
            }
            ui.label(
                None,
                &format!(
                    "Progress: level {} of {} unlocked",
                    state.highest_unlocked_level + 1,
                    state.levels_len()
                ),
            );
            ui.label(
                None,
                if state.game.paused {
                    "Flight state: Paused"
                } else {
                    "Flight state: Active"
                },
            );
        });

    actions
}

pub(crate) fn phase_text(phase: GamePhase, paused: bool) -> &'static str {
    match phase {
        GamePhase::Aiming => "Aiming",
        GamePhase::Flying => {
            if paused {
                "Paused"
            } else {
                "Flying"
            }
        }
        GamePhase::Success => "Success",
        GamePhase::Failed => "Failed",
    }
}
