use macroquad::prelude::*;

use crate::constants::{CONTROLS_Y, TITLE_Y};
use crate::controls::phase_text;
use crate::model::{GamePhase, Prediction, StepOutcome};
use crate::physics::world_to_screen;
use crate::render::draw_ui_text;
use crate::state::AppRuntime;

pub(crate) fn draw_hud(
    state: &AppRuntime,
    prediction: &Prediction,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    screen_h: f32,
    font: Option<&Font>,
) -> bool {
    draw_range_label(
        prediction,
        world_max_x,
        world_max_y,
        left,
        right,
        top,
        bottom,
        font,
    );
    draw_header_block(state, left, right, font);
    draw_status_block(state, prediction, left, screen_h, font);
    draw_prediction_hint(prediction.outcome, right, top, font);
    draw_next_level_button(state, right, font)
}

fn draw_range_label(
    prediction: &Prediction,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    font: Option<&Font>,
) {
    let range_label = format!("{:.2} m", prediction.range_m.max(0.0));
    let range_label_size = measure_text(&range_label, font, 18, 1.0);
    let landing_point = world_to_screen(
        vec2(prediction.range_m.max(0.0), 0.0),
        world_max_x,
        world_max_y,
        left,
        right,
        top,
        bottom,
    );
    let label_x = (landing_point.x - (range_label_size.width * 0.5))
        .clamp(left + 4.0, right - range_label_size.width - 4.0);
    let label_y = (bottom - 12.0).max(top + 20.0);
    draw_ui_text(&range_label, label_x, label_y, 18, DARKGRAY, font);
}

fn draw_header_block(state: &AppRuntime, left: f32, right: f32, font: Option<&Font>) {
    let header_color = Color::from_rgba(30, 30, 35, 255);
    draw_ui_text(
        "ParabolicRust - Interactive Game Mode",
        left,
        TITLE_Y,
        30,
        header_color,
        font,
    );
    draw_ui_text(
        &format!(
            "Level: {} - {} ({})",
            state.current_level().code,
            state.current_level().title,
            state.current_level().environment.name
        ),
        left,
        TITLE_Y + 30.0,
        22,
        DARKGRAY,
        font,
    );
    let top_right_level = format!(
        "{} - Level {}",
        state.current_level().environment.name,
        state.current_level().level_in_environment
    );
    let top_right_size = measure_text(&top_right_level, font, 24, 1.0);
    draw_ui_text(
        &top_right_level,
        right - top_right_size.width,
        TITLE_Y + 2.0,
        24,
        DARKGRAY,
        font,
    );
    draw_ui_text(
        "Controls: drag launch dot left/up/down for angle+speed | W/S height | A/D velocity | Space launch/pause | R reset | P/N level nav",
        left + 12.0,
        CONTROLS_Y,
        20,
        DARKGRAY,
        font,
    );
}

fn draw_status_block(
    state: &AppRuntime,
    prediction: &Prediction,
    left: f32,
    screen_h: f32,
    font: Option<&Font>,
) {
    let header_color = Color::from_rgba(30, 30, 35, 255);
    let active_time = state.game.shot.map_or(0.0, |s| s.elapsed_s);
    let active_range = state.game.shot.map_or(0.0, |s| s.position.x.max(0.0));
    let active_bounces = state.game.shot.map_or(0, |s| s.bounces);
    let phase = phase_text(state.game.phase, state.game.paused);

    draw_ui_text(
        &format!(
            "Angle: {:.1} deg | Velocity: {:.1} m/s | Height: {:.1} m",
            state.config.angle_deg, state.config.speed_mps, state.config.height_m
        ),
        left,
        screen_h - 45.0,
        24,
        header_color,
        font,
    );
    draw_ui_text(
        &format!(
            "Flight: {:.2} s | Range: {:.2} m | Bounces: {} | Speed x{:.2} | State: {}",
            active_time, active_range, active_bounces, state.sim_speed, phase
        ),
        left,
        screen_h - 14.0,
        20,
        BLUE,
        font,
    );
    draw_ui_text(
        &format!(
            "Prediction -> range {:.2} m, flight {:.2} s, bounces {} | {}",
            prediction.range_m,
            prediction.flight_time_s,
            prediction.bounces,
            state.game.status_line
        ),
        left,
        screen_h - 76.0,
        18,
        DARKGRAY,
        font,
    );
}

fn draw_prediction_hint(outcome: StepOutcome, right: f32, top: f32, font: Option<&Font>) {
    if outcome == StepOutcome::HitTarget {
        draw_ui_text(
            "Preview says: valid hit path found",
            right - 330.0,
            top + 22.0,
            18,
            DARKGREEN,
            font,
        );
    }
}

fn draw_next_level_button(state: &AppRuntime, right: f32, font: Option<&Font>) -> bool {
    let has_next_level = state.current_level_idx + 1 < state.levels_len();
    let next_unlocked = state.current_level_idx < state.highest_unlocked_level;
    let visible = state.game.phase == GamePhase::Success && has_next_level && next_unlocked;
    if !visible {
        return false;
    }

    let button_w = 230.0;
    let button_h = 52.0;
    let button_x = right - button_w;
    let button_y = TITLE_Y + 38.0;
    let button_rect = Rect::new(button_x, button_y, button_w, button_h);

    let mouse = mouse_position();
    let mouse_v = vec2(mouse.0, mouse.1);
    let hovered = button_rect.contains(mouse_v);
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let fill = if hovered {
        Color::from_rgba(37, 99, 235, 255)
    } else {
        Color::from_rgba(29, 78, 216, 255)
    };
    draw_rectangle(
        button_rect.x,
        button_rect.y,
        button_rect.w,
        button_rect.h,
        fill,
    );
    draw_rectangle_lines(
        button_rect.x,
        button_rect.y,
        button_rect.w,
        button_rect.h,
        2.5,
        WHITE,
    );

    let label = "Next Level";
    let size = measure_text(label, font, 30, 1.0);
    draw_ui_text(
        label,
        button_rect.x + ((button_rect.w - size.width) * 0.5),
        button_rect.y + ((button_rect.h + size.height) * 0.5) - 5.0,
        30,
        WHITE,
        font,
    );

    clicked
}
