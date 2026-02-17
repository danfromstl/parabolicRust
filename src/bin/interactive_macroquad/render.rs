use macroquad::prelude::*;

use crate::constants::{
    LAUNCH_GHOST_RADIUS, LAUNCH_HANDLE_RADIUS, ROTATE_HANDLE_RADIUS, START_BUTTON_COLOR,
    START_BUTTON_TEXT, SURFACE_HANDLE_RADIUS, TITLE_SCREEN_BG, X_GRID_LINES, Y_GRID_LINES,
};
use crate::model::{LaunchEditor, Level, SurfaceEditor};
use crate::physics::{bounce_surface_edges, rotation_handle_screen, world_to_screen};

fn format_axis_value(value: f32, axis_max: f32) -> String {
    if axis_max >= 1000.0 {
        format!("{value:.0}")
    } else if axis_max >= 100.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

pub(crate) fn draw_ui_text(
    text: &str,
    x: f32,
    y: f32,
    font_size: u16,
    color: Color,
    font: Option<&Font>,
) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font,
            font_size,
            color,
            ..Default::default()
        },
    );
}

pub(crate) fn draw_grid(left: f32, right: f32, top: f32, bottom: f32, color: Color) {
    for i in 0..=X_GRID_LINES {
        let t = i as f32 / X_GRID_LINES as f32;
        let x = left + t * (right - left);
        draw_line(x, top, x, bottom, 1.0, color);
    }
    for i in 0..=Y_GRID_LINES {
        let t = i as f32 / Y_GRID_LINES as f32;
        let y = bottom - t * (bottom - top);
        draw_line(left, y, right, y, 1.0, color);
    }
}

pub(crate) fn draw_axis_tick_labels(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    world_max_x: f32,
    world_max_y: f32,
    font: Option<&Font>,
) {
    let label_color = Color::from_rgba(105, 113, 124, 255);
    let tick_font_size: u16 = 16;

    for i in 0..=X_GRID_LINES {
        let t = i as f32 / X_GRID_LINES as f32;
        let x = left + t * (right - left);
        let value = t * world_max_x;
        let label = format_axis_value(value, world_max_x);
        let size = measure_text(&label, font, tick_font_size, 1.0);
        draw_ui_text(
            &label,
            x - (size.width * 0.5),
            bottom + 22.0,
            tick_font_size,
            label_color,
            font,
        );
    }

    for i in 0..=Y_GRID_LINES {
        let t = i as f32 / Y_GRID_LINES as f32;
        let y = bottom - t * (bottom - top);
        let value = t * world_max_y;
        let label = format_axis_value(value, world_max_y);
        let size = measure_text(&label, font, tick_font_size, 1.0);
        draw_ui_text(
            &label,
            (left - 8.0) - size.width,
            y + (size.height * 0.35),
            tick_font_size,
            label_color,
            font,
        );
    }

    draw_ui_text(
        "Distance (m)",
        right - 130.0,
        bottom + 48.0,
        18,
        label_color,
        font,
    );
    draw_ui_text("Height (m)", left + 10.0, top - 8.0, 18, label_color, font);
}

pub(crate) fn draw_level_objects(
    level: &Level,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    show_surface_handles: bool,
    editor: &SurfaceEditor,
) {
    if let Some(surface) = level.bounce_surface {
        let corners = surface.corners.map(|corner| {
            world_to_screen(corner, world_max_x, world_max_y, left, right, top, bottom)
        });
        draw_triangle(
            corners[0],
            corners[1],
            corners[2],
            Color::from_rgba(242, 159, 5, 115),
        );
        draw_triangle(
            corners[0],
            corners[2],
            corners[3],
            Color::from_rgba(242, 159, 5, 115),
        );

        for (a, b) in bounce_surface_edges(&corners) {
            draw_line(a.x, a.y, b.x, b.y, 3.0, Color::from_rgba(242, 159, 5, 255));
        }

        if show_surface_handles || editor.is_dragging() {
            let active_corner = editor.active_corner();
            for (idx, corner) in corners.iter().copied().enumerate() {
                let is_active = active_corner == Some(idx);
                let is_hovered = editor.hovered_corner == Some(idx);
                draw_circle(
                    corner.x,
                    corner.y,
                    SURFACE_HANDLE_RADIUS,
                    if is_active {
                        Color::from_rgba(37, 99, 235, 255)
                    } else if is_hovered {
                        Color::from_rgba(186, 214, 255, 255)
                    } else {
                        WHITE
                    },
                );
                draw_circle_lines(
                    corner.x,
                    corner.y,
                    SURFACE_HANDLE_RADIUS,
                    2.0,
                    Color::from_rgba(32, 32, 36, 255),
                );
            }

            let (rotate_anchor, rotate_handle) = rotation_handle_screen(&corners);
            draw_line(
                rotate_anchor.x,
                rotate_anchor.y,
                rotate_handle.x,
                rotate_handle.y,
                2.0,
                Color::from_rgba(92, 99, 112, 255),
            );
            draw_circle(
                rotate_handle.x,
                rotate_handle.y,
                ROTATE_HANDLE_RADIUS,
                if editor.active_rotate() {
                    Color::from_rgba(37, 99, 235, 255)
                } else if editor.hovered_rotate {
                    Color::from_rgba(186, 214, 255, 255)
                } else {
                    WHITE
                },
            );
            draw_circle_lines(
                rotate_handle.x,
                rotate_handle.y,
                ROTATE_HANDLE_RADIUS,
                2.0,
                Color::from_rgba(32, 32, 36, 255),
            );
        }
    }

    let target_center = world_to_screen(
        level.target.center,
        world_max_x,
        world_max_y,
        left,
        right,
        top,
        bottom,
    );
    let px_per_world_x = (right - left) / world_max_x.max(1.0);
    let px_per_world_y = (bottom - top) / world_max_y.max(1.0);
    let target_radius_px = (level.target.radius_m * px_per_world_x.min(px_per_world_y)).max(4.0);
    draw_circle(
        target_center.x,
        target_center.y,
        target_radius_px,
        Color::from_rgba(81, 201, 122, 220),
    );
    draw_circle_lines(
        target_center.x,
        target_center.y,
        target_radius_px,
        2.0,
        DARKGREEN,
    );

    for barrier in &level.barriers {
        let top_left_world = vec2(barrier.rect.x, barrier.rect.y + barrier.rect.h);
        let bottom_right_world = vec2(barrier.rect.x + barrier.rect.w, barrier.rect.y);
        let top_left = world_to_screen(
            top_left_world,
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        let bottom_right = world_to_screen(
            bottom_right_world,
            world_max_x,
            world_max_y,
            left,
            right,
            top,
            bottom,
        );
        draw_rectangle(
            top_left.x,
            top_left.y,
            (bottom_right.x - top_left.x).max(2.0),
            (bottom_right.y - top_left.y).max(2.0),
            Color::from_rgba(170, 84, 84, 220),
        );
    }
}

pub(crate) fn draw_path(
    points: &[Vec2],
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    thickness: f32,
    color: Color,
) {
    if points.len() < 2 {
        return;
    }
    let mut prev = world_to_screen(
        points[0],
        world_max_x,
        world_max_y,
        left,
        right,
        top,
        bottom,
    );
    for point in points.iter().skip(1).copied() {
        let cur = world_to_screen(point, world_max_x, world_max_y, left, right, top, bottom);
        draw_line(prev.x, prev.y, cur.x, cur.y, thickness, color);
        prev = cur;
    }
}

pub(crate) fn draw_launch_editor(launch_screen: Vec2, editor: &LaunchEditor) {
    let launch_fill = if editor.active {
        Color::from_rgba(37, 99, 235, 255)
    } else if editor.hovered {
        Color::from_rgba(191, 219, 254, 255)
    } else {
        Color::from_rgba(245, 89, 89, 255)
    };
    draw_circle(
        launch_screen.x,
        launch_screen.y,
        LAUNCH_HANDLE_RADIUS,
        launch_fill,
    );
    draw_circle_lines(
        launch_screen.x,
        launch_screen.y,
        LAUNCH_HANDLE_RADIUS,
        2.0,
        Color::from_rgba(121, 28, 28, 255),
    );

    if editor.active {
        draw_line(
            launch_screen.x,
            launch_screen.y,
            editor.ghost_screen.x,
            editor.ghost_screen.y,
            2.0,
            Color::from_rgba(37, 99, 235, 215),
        );
        draw_circle(
            editor.ghost_screen.x,
            editor.ghost_screen.y,
            LAUNCH_GHOST_RADIUS,
            Color::from_rgba(37, 99, 235, 230),
        );
        draw_circle_lines(
            editor.ghost_screen.x,
            editor.ghost_screen.y,
            LAUNCH_GHOST_RADIUS,
            2.0,
            WHITE,
        );
    }
}

pub(crate) fn draw_title_screen(screen_w: f32, screen_h: f32, font: Option<&Font>) -> bool {
    clear_background(TITLE_SCREEN_BG);

    let title = "Parabolic Rust";
    let title_size: u16 = 110;
    let title_measure = measure_text(title, font, title_size, 1.0);
    let title_x = (screen_w - title_measure.width) * 0.5;
    let title_y = (screen_h * 0.40).max(180.0);
    draw_ui_text(title, title_x, title_y, title_size, BLACK, font);

    let button_w = 330.0;
    let button_h = 90.0;
    let button_x = (screen_w - button_w) * 0.5;
    let button_y = title_y + 70.0;
    let button_rect = Rect::new(button_x, button_y, button_w, button_h);
    draw_rectangle(
        button_rect.x,
        button_rect.y,
        button_rect.w,
        button_rect.h,
        START_BUTTON_COLOR,
    );
    draw_rectangle_lines(
        button_rect.x,
        button_rect.y,
        button_rect.w,
        button_rect.h,
        3.0,
        WHITE,
    );

    let button_text_size: u16 = 42;
    let button_text_measure = measure_text(START_BUTTON_TEXT, font, button_text_size, 1.0);
    draw_ui_text(
        START_BUTTON_TEXT,
        button_rect.x + (button_rect.w - button_text_measure.width) * 0.5,
        button_rect.y + (button_rect.h + button_text_measure.height) * 0.5 - 8.0,
        button_text_size,
        WHITE,
        font,
    );

    let hint = "Click button or press Enter/Space";
    let hint_size: u16 = 22;
    let hint_measure = measure_text(hint, font, hint_size, 1.0);
    draw_ui_text(
        hint,
        (screen_w - hint_measure.width) * 0.5,
        button_rect.y + button_rect.h + 38.0,
        hint_size,
        DARKGRAY,
        font,
    );

    let mouse = mouse_position();
    let mouse_vec = vec2(mouse.0, mouse.1);
    let clicked_start =
        is_mouse_button_pressed(MouseButton::Left) && button_rect.contains(mouse_vec);
    let hotkey_start = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);

    clicked_start || hotkey_start
}
