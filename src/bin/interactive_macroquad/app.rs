use macroquad::prelude::*;

use crate::constants::{
    BOTTOM_MARGIN, INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH, LEFT_MARGIN, MSAA_SAMPLES,
    RIGHT_MARGIN, TOP_MARGIN, UI_FONT_PATH,
};
use crate::controls::{FrameActions, draw_control_panel, hotkey_actions};
use crate::gameplay::{apply_actions, apply_keyboard_adjustments, step_active_flight};
use crate::hud::draw_hud;
use crate::input::{update_launch_editor, update_surface_editor};
use crate::model::AppScene;
use crate::physics::{compute_world_window, simulate_prediction, world_to_screen};
use crate::render::{
    draw_axis_tick_labels, draw_grid, draw_launch_editor, draw_level_objects, draw_path,
    draw_title_screen,
};
use crate::state::AppRuntime;

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

    let mut state = AppRuntime::new();

    loop {
        let frame_dt = get_frame_time();
        let screen_w = screen_width();
        let screen_h = screen_height();

        if state.scene == AppScene::Title {
            if draw_title_screen(screen_w, screen_h, ui_font.as_ref()) {
                state.scene = AppScene::Game;
                state.game.reset();
                state.set_loaded_status();
            }
            next_frame().await;
            continue;
        }

        let left = LEFT_MARGIN;
        let right = screen_w - RIGHT_MARGIN;
        let top = TOP_MARGIN;
        let bottom = screen_h - BOTTOM_MARGIN;

        let actions = hotkey_actions().merge(draw_control_panel(&mut state));
        if apply_actions(&mut state, actions) {
            continue;
        }

        apply_keyboard_adjustments(&mut state, frame_dt);
        step_active_flight(&mut state, frame_dt);

        let mut prediction = simulate_prediction(state.config, state.current_level());
        let (mut world_max_x, mut world_max_y) = compute_world_window(
            state.current_level(),
            state.config,
            &prediction,
            state.game.shot,
        );

        let mouse = mouse_position();
        let mouse_screen = vec2(mouse.0, mouse.1);
        let launch_screen = world_to_screen(
            vec2(0.0, state.config.height_m.max(0.0)),
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        let launch_drag_changed = update_launch_editor(
            &mut state.config,
            &mut state.launch_editor,
            state.game.phase,
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
            prediction = simulate_prediction(state.config, state.current_level());
            let window = compute_world_window(
                state.current_level(),
                state.config,
                &prediction,
                state.game.shot,
            );
            world_max_x = window.0;
            world_max_y = window.1;
        }

        let level_idx = state.current_level_idx;
        let show_surface_handles = update_surface_editor(
            &mut state.levels[level_idx],
            &mut state.surface_editor,
            mouse_screen,
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );

        if show_surface_handles || state.surface_editor.is_dragging() {
            prediction = simulate_prediction(state.config, state.current_level());
            let window = compute_world_window(
                state.current_level(),
                state.config,
                &prediction,
                state.game.shot,
            );
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
            state.current_level(),
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
            show_surface_handles,
            &state.surface_editor,
        );
        let launch_screen_after = world_to_screen(
            vec2(0.0, state.config.height_m.max(0.0)),
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        draw_launch_editor(launch_screen_after, &state.launch_editor);

        if state.show_preview && !matches!(state.game.phase, crate::model::GamePhase::Flying) {
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

        if !state.game.trail.is_empty() {
            draw_path(
                &state.game.trail,
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

        if let Some(shot) = state.game.shot {
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

        let next_level_button_clicked = draw_hud(
            &state,
            &prediction,
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
            screen_h,
            ui_font.as_ref(),
        );

        if next_level_button_clicked
            && apply_actions(
                &mut state,
                FrameActions {
                    next_level: true,
                    ..Default::default()
                },
            )
        {
            continue;
        }

        next_frame().await;
    }
}
