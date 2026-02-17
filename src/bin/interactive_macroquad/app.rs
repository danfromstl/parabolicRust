use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use crate::constants::{
    BOTTOM_MARGIN, CONTROLS_Y, FIXED_STEP_S, HEIGHT_KEY_RATE_MPS, INITIAL_WINDOW_HEIGHT,
    INITIAL_WINDOW_WIDTH, LEFT_MARGIN, MAX_SIM_TIME_S, MSAA_SAMPLES, RIGHT_MARGIN, TITLE_Y,
    TOP_MARGIN, UI_FONT_PATH, VELOCITY_KEY_RATE_MPS,
};
use crate::input::{update_launch_editor, update_surface_editor};
use crate::model::{
    AppScene, GamePhase, GameState, LaunchEditor, Level, StepOutcome, SurfaceEditor,
};
use crate::physics::{compute_world_window, simulate_prediction, step_projectile, world_to_screen};
use crate::render::{
    draw_axis_tick_labels, draw_grid, draw_launch_editor, draw_level_objects, draw_path,
    draw_title_screen, draw_ui_text,
};

pub(crate) fn window_conf() -> Conf {
    Conf {
        window_title: "ParabolicRust Interactive".to_string(),
        window_width: INITIAL_WINDOW_WIDTH,
        window_height: INITIAL_WINDOW_HEIGHT,
        high_dpi: true,
        sample_count: MSAA_SAMPLES,
        ..Default::default()
    }
}

pub(crate) async fn run() {
    let ui_font = match load_ttf_font(UI_FONT_PATH).await {
        Ok(font) => Some(font),
        Err(err) => {
            println!("Could not load '{UI_FONT_PATH}': {err}. Falling back to default font.");
            None
        }
    };

    let mut levels = Level::moon_campaign();
    let mut current_level_idx = 0usize;
    let mut highest_unlocked_level = 0usize;
    let mut config = levels[current_level_idx].default_launch;
    let mut game = GameState::new();
    let mut show_preview = true;
    let mut sim_speed = 1.0f32;
    let mut scene = AppScene::Title;
    let mut surface_editor = SurfaceEditor::new();
    let mut launch_editor = LaunchEditor::new();

    loop {
        let frame_dt = get_frame_time();
        let screen_w = screen_width();
        let screen_h = screen_height();

        if scene == AppScene::Title {
            if draw_title_screen(screen_w, screen_h, ui_font.as_ref()) {
                scene = AppScene::Game;
                game.reset();
                game.status_line = format!("Loaded {}", levels[current_level_idx].code);
            }
            next_frame().await;
            continue;
        }

        let left = LEFT_MARGIN;
        let right = screen_w - RIGHT_MARGIN;
        let top = TOP_MARGIN;
        let bottom = screen_h - BOTTOM_MARGIN;

        if is_key_pressed(KeyCode::Space) {
            match game.phase {
                GamePhase::Flying => {
                    game.paused = !game.paused;
                    game.status_line = if game.paused {
                        "Paused".to_string()
                    } else {
                        "Resumed".to_string()
                    };
                }
                _ => game.launch(config),
            }
        }
        if is_key_pressed(KeyCode::R) {
            game.reset();
        }
        if is_key_pressed(KeyCode::N) && current_level_idx < highest_unlocked_level {
            current_level_idx += 1;
            config = levels[current_level_idx].default_launch;
            game.reset();
            game.status_line = format!("Advanced to {}", levels[current_level_idx].code);
        }
        if is_key_pressed(KeyCode::P) && current_level_idx > 0 {
            current_level_idx -= 1;
            config = levels[current_level_idx].default_launch;
            game.reset();
            game.status_line = format!("Moved to {}", levels[current_level_idx].code);
        }

        let level_code = levels[current_level_idx].code;
        let level_env = levels[current_level_idx].environment;

        // UI panel with sliders/buttons for "real" controls.
        let mut launch_clicked = false;
        let mut reset_clicked = false;
        let mut next_level_clicked = false;
        let mut prev_level_clicked = false;
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
                ui.slider(hash!(), "Angle (deg)", -89.0..89.0, &mut config.angle_deg);
                ui.slider(hash!(), "Velocity (m/s)", 5.0..500.0, &mut config.speed_mps);
                ui.slider(hash!(), "Height (m)", 0.0..400.0, &mut config.height_m);
                ui.slider(hash!(), "Simulation Speed", 0.5..5.0, &mut sim_speed);
                ui.separator();
                if ui.button(None, "Launch (Space)") {
                    launch_clicked = true;
                }
                if ui.button(None, "Reset (R)") {
                    reset_clicked = true;
                }
                if ui.button(None, "Toggle Preview") {
                    show_preview = !show_preview;
                }
                if ui.button(None, "Prev Level (P)") {
                    prev_level_clicked = true;
                }
                if ui.button(None, "Next Level (N)") {
                    next_level_clicked = true;
                }
                ui.label(
                    None,
                    &format!(
                        "Progress: level {} of {} unlocked",
                        highest_unlocked_level + 1,
                        levels.len()
                    ),
                );
                ui.label(
                    None,
                    if game.paused {
                        "Flight state: Paused"
                    } else {
                        "Flight state: Active"
                    },
                );
            });

        if launch_clicked {
            match game.phase {
                GamePhase::Flying => {
                    game.paused = !game.paused;
                    game.status_line = if game.paused {
                        "Paused".to_string()
                    } else {
                        "Resumed".to_string()
                    };
                }
                _ => game.launch(config),
            }
        }
        if reset_clicked {
            game.reset();
        }
        if prev_level_clicked && current_level_idx > 0 {
            current_level_idx -= 1;
            config = levels[current_level_idx].default_launch;
            game.reset();
            game.status_line = format!("Moved to {}", levels[current_level_idx].code);
            continue;
        }
        if next_level_clicked && current_level_idx < highest_unlocked_level {
            current_level_idx += 1;
            config = levels[current_level_idx].default_launch;
            game.reset();
            game.status_line = format!("Advanced to {}", levels[current_level_idx].code);
            continue;
        }

        if !is_mouse_button_down(MouseButton::Left) {
            if is_key_down(KeyCode::W) {
                config.height_m += HEIGHT_KEY_RATE_MPS * frame_dt;
            }
            if is_key_down(KeyCode::S) {
                config.height_m -= HEIGHT_KEY_RATE_MPS * frame_dt;
            }
            if is_key_down(KeyCode::D) {
                config.speed_mps += VELOCITY_KEY_RATE_MPS * frame_dt;
            }
            if is_key_down(KeyCode::A) {
                config.speed_mps -= VELOCITY_KEY_RATE_MPS * frame_dt;
            }
        }
        config.height_m = config.height_m.clamp(0.0, 400.0);
        config.speed_mps = config.speed_mps.clamp(5.0, 500.0);

        if matches!(game.phase, GamePhase::Flying) && !game.paused {
            let mut remaining = (frame_dt * sim_speed).min(0.10);
            while remaining > 0.0 {
                let dt = remaining.min(FIXED_STEP_S);
                remaining -= dt;

                if let Some(shot) = game.shot.as_mut() {
                    let outcome = step_projectile(shot, &levels[current_level_idx], dt);
                    game.trail.push(shot.position);

                    if outcome != StepOutcome::Flying {
                        game.phase = if outcome == StepOutcome::HitTarget {
                            GamePhase::Success
                        } else {
                            GamePhase::Failed
                        };
                        game.status_line = match outcome {
                            StepOutcome::HitTarget => {
                                if current_level_idx == highest_unlocked_level
                                    && highest_unlocked_level + 1 < levels.len()
                                {
                                    highest_unlocked_level += 1;
                                }
                                let unlock_note = if current_level_idx < highest_unlocked_level {
                                    " | Next level unlocked (N)"
                                } else {
                                    " | Campaign complete"
                                };
                                format!(
                                    "Target hit in {:.2}s with {} bounce(s){}",
                                    shot.elapsed_s, shot.bounces, unlock_note
                                )
                            }
                            StepOutcome::HitGround => {
                                format!(
                                    "Missed target: hit ground at x={:.2} m after {} bounce(s)",
                                    shot.position.x.max(0.0),
                                    shot.bounces
                                )
                            }
                            StepOutcome::HitBarrier => {
                                "Missed target: barrier collision".to_string()
                            }
                            StepOutcome::Flying => String::new(),
                        };
                        break;
                    }

                    if shot.elapsed_s > MAX_SIM_TIME_S {
                        game.phase = GamePhase::Failed;
                        game.status_line = "Missed target: timed out".to_string();
                        break;
                    }
                }
            }
        }

        let mut prediction = simulate_prediction(config, &levels[current_level_idx]);
        let (mut world_max_x, mut world_max_y) =
            compute_world_window(&levels[current_level_idx], config, &prediction, game.shot);

        let mouse = mouse_position();
        let mouse_screen = vec2(mouse.0, mouse.1);
        let launch_screen = world_to_screen(
            vec2(0.0, config.height_m.max(0.0)),
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        let launch_drag_changed = update_launch_editor(
            &mut config,
            &mut launch_editor,
            game.phase,
            mouse_screen,
            launch_screen,
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        if launch_drag_changed {
            prediction = simulate_prediction(config, &levels[current_level_idx]);
            let window =
                compute_world_window(&levels[current_level_idx], config, &prediction, game.shot);
            world_max_x = window.0;
            world_max_y = window.1;
        }

        let show_surface_handles = update_surface_editor(
            &mut levels[current_level_idx],
            &mut surface_editor,
            mouse_screen,
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );

        if show_surface_handles || surface_editor.is_dragging() {
            prediction = simulate_prediction(config, &levels[current_level_idx]);
            let window =
                compute_world_window(&levels[current_level_idx], config, &prediction, game.shot);
            world_max_x = window.0;
            world_max_y = window.1;
        }

        clear_background(Color::from_rgba(250, 251, 253, 255));
        draw_grid(
            left,
            right,
            top,
            bottom,
            Color::from_rgba(227, 231, 236, 255),
        );
        draw_line(left, bottom, right, bottom, 2.0, DARKGRAY);
        draw_line(left, top, left, bottom, 2.0, DARKGRAY);
        draw_axis_tick_labels(
            left,
            right,
            top,
            bottom,
            world_max_x,
            world_max_y,
            ui_font.as_ref(),
        );
        draw_level_objects(
            &levels[current_level_idx],
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
            show_surface_handles,
            &surface_editor,
        );
        let launch_screen_after = world_to_screen(
            vec2(0.0, config.height_m.max(0.0)),
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        draw_launch_editor(launch_screen_after, &launch_editor);

        if show_preview && !matches!(game.phase, GamePhase::Flying) {
            draw_path(
                &prediction.points,
                world_max_x,
                world_max_y,
                left,
                right,
                top,
                bottom,
                2.0,
                Color::from_rgba(76, 141, 245, 140),
            );
        }

        if !game.trail.is_empty() {
            draw_path(
                &game.trail,
                world_max_x,
                world_max_y,
                left,
                right,
                top,
                bottom,
                3.0,
                Color::from_rgba(54, 123, 245, 255),
            );
        }

        if let Some(shot) = game.shot {
            let p = world_to_screen(
                shot.position,
                world_max_x,
                world_max_y,
                left,
                right,
                top,
                bottom,
            );
            draw_circle(p.x, p.y, 7.0, RED);
            draw_circle_lines(p.x, p.y, 7.0, 2.0, MAROON);
        }

        // Keep duplicated range label near x-axis intersection.
        let range_label = format!("{:.2} m", prediction.range_m.max(0.0));
        let range_label_size = measure_text(&range_label, ui_font.as_ref(), 18, 1.0);
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
        draw_ui_text(
            &range_label,
            label_x,
            label_y,
            18,
            DARKGRAY,
            ui_font.as_ref(),
        );

        // Header + status lines.
        let header_color = Color::from_rgba(30, 30, 35, 255);
        draw_ui_text(
            "ParabolicRust - Interactive Game Mode",
            left,
            TITLE_Y,
            30,
            header_color,
            ui_font.as_ref(),
        );
        draw_ui_text(
            &format!(
                "Level: {} - {} ({})",
                levels[current_level_idx].code,
                levels[current_level_idx].title,
                levels[current_level_idx].environment.name
            ),
            left,
            TITLE_Y + 30.0,
            22,
            DARKGRAY,
            ui_font.as_ref(),
        );
        let top_right_level = format!(
            "{} - Level {}",
            levels[current_level_idx].environment.name,
            current_level_idx + 1
        );
        let top_right_size = measure_text(&top_right_level, ui_font.as_ref(), 24, 1.0);
        draw_ui_text(
            &top_right_level,
            right - top_right_size.width,
            TITLE_Y + 2.0,
            24,
            DARKGRAY,
            ui_font.as_ref(),
        );
        draw_ui_text(
            "Controls: drag launch dot left/up/down for angle+speed | W/S height | A/D velocity | Space launch/pause | R reset | P/N level nav",
            left + 12.0,
            CONTROLS_Y,
            20,
            DARKGRAY,
            ui_font.as_ref(),
        );

        let active_time = game.shot.map_or(0.0, |s| s.elapsed_s);
        let active_range = game.shot.map_or(0.0, |s| s.position.x.max(0.0));
        let active_bounces = game.shot.map_or(0, |s| s.bounces);
        let phase_text = match game.phase {
            GamePhase::Aiming => "Aiming",
            GamePhase::Flying => {
                if game.paused {
                    "Paused"
                } else {
                    "Flying"
                }
            }
            GamePhase::Success => "Success",
            GamePhase::Failed => "Failed",
        };
        draw_ui_text(
            &format!(
                "Angle: {:.1} deg | Velocity: {:.1} m/s | Height: {:.1} m",
                config.angle_deg, config.speed_mps, config.height_m
            ),
            left,
            screen_h - 45.0,
            24,
            header_color,
            ui_font.as_ref(),
        );
        draw_ui_text(
            &format!(
                "Flight: {:.2} s | Range: {:.2} m | Bounces: {} | Speed x{:.2} | State: {}",
                active_time, active_range, active_bounces, sim_speed, phase_text
            ),
            left,
            screen_h - 14.0,
            20,
            BLUE,
            ui_font.as_ref(),
        );
        draw_ui_text(
            &format!(
                "Prediction -> range {:.2} m, flight {:.2} s, bounces {} | {}",
                prediction.range_m, prediction.flight_time_s, prediction.bounces, game.status_line
            ),
            left,
            screen_h - 76.0,
            18,
            DARKGRAY,
            ui_font.as_ref(),
        );

        if prediction.outcome == StepOutcome::HitTarget {
            draw_ui_text(
                "Preview says: valid hit path found",
                right - 330.0,
                top + 22.0,
                18,
                DARKGREEN,
                ui_font.as_ref(),
            );
        }

        next_frame().await;
    }
}
