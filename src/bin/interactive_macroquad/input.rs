use macroquad::prelude::*;

use crate::constants::{
    LAUNCH_DRAG_MIN_PX, LAUNCH_GHOST_BELOW_AXIS_PX, LAUNCH_HANDLE_RADIUS, ROTATE_HANDLE_RADIUS,
    SLINGSHOT_VERTICAL_MIRROR, SURFACE_HANDLE_RADIUS,
};
use crate::model::{GamePhase, LaunchConfig, LaunchEditor, Level, SurfaceDragMode, SurfaceEditor};
use crate::physics::{
    point_in_polygon, quad_center, rotate_vec, rotation_handle_screen, screen_to_world,
    world_to_screen,
};

pub(crate) fn update_surface_editor(
    level: &mut Level,
    editor: &mut SurfaceEditor,
    mouse_screen: Vec2,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> bool {
    let Some(surface) = level.bounce_surface.as_mut() else {
        editor.drag_mode = None;
        editor.hovered_corner = None;
        editor.hovered_surface = false;
        editor.hovered_rotate = false;
        return false;
    };

    if !is_mouse_button_down(MouseButton::Left) {
        editor.drag_mode = None;
    }

    let corners_screen = surface
        .corners
        .map(|corner| world_to_screen(corner, world_max_x, world_max_y, left, right, top, bottom));

    let mut hovered_corner = None;
    for (idx, corner) in corners_screen.iter().copied().enumerate() {
        if mouse_screen.distance(corner) <= (SURFACE_HANDLE_RADIUS + 4.0) {
            hovered_corner = Some(idx);
            break;
        }
    }

    let (_rotate_anchor, rotate_handle) = rotation_handle_screen(&corners_screen);
    let hovered_rotate = mouse_screen.distance(rotate_handle) <= (ROTATE_HANDLE_RADIUS + 4.0);
    let hovered_inside = point_in_polygon(mouse_screen, &corners_screen);
    let hovered_surface = hovered_corner.is_some() || hovered_inside || hovered_rotate;

    editor.hovered_corner = hovered_corner;
    editor.hovered_rotate = hovered_rotate;
    editor.hovered_surface = hovered_surface;

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(corner_idx) = hovered_corner {
            editor.drag_mode = Some(SurfaceDragMode::Corner(corner_idx));
        } else if hovered_rotate {
            editor.drag_mode = Some(SurfaceDragMode::Rotate);
            editor.drag_start_corners = surface.corners;
            editor.rotate_center_world = quad_center(&surface.corners);
            let center_screen = world_to_screen(
                editor.rotate_center_world,
                world_max_x,
                world_max_y,
                left,
                right,
                top,
                bottom,
            );
            editor.rotate_start_angle_rad =
                (mouse_screen.y - center_screen.y).atan2(mouse_screen.x - center_screen.x);
        } else if hovered_inside {
            editor.drag_mode = Some(SurfaceDragMode::Surface);
            editor.drag_start_mouse_world = screen_to_world(
                mouse_screen,
                world_max_x,
                world_max_y,
                left,
                right,
                top,
                bottom,
            );
            editor.drag_start_corners = surface.corners;
        }
    }

    let mut changed = false;
    if let Some(mode) = editor.drag_mode {
        match mode {
            SurfaceDragMode::Corner(corner_idx) => {
                let mut world = screen_to_world(
                    mouse_screen,
                    world_max_x,
                    world_max_y,
                    left,
                    right,
                    top,
                    bottom,
                );
                // Keep corners within currently visible positive world.
                world.x = world.x.clamp(0.0, world_max_x.max(1.0));
                world.y = world.y.clamp(0.0, world_max_y.max(1.0));
                surface.corners[corner_idx] = world;
                changed = true;
            }
            SurfaceDragMode::Surface => {
                let current_world = screen_to_world(
                    mouse_screen,
                    world_max_x,
                    world_max_y,
                    left,
                    right,
                    top,
                    bottom,
                );
                let mut delta = current_world - editor.drag_start_mouse_world;
                let mut min_x = editor.drag_start_corners[0].x;
                let mut max_x = editor.drag_start_corners[0].x;
                let mut min_y = editor.drag_start_corners[0].y;
                let mut max_y = editor.drag_start_corners[0].y;
                for corner in &editor.drag_start_corners[1..] {
                    min_x = min_x.min(corner.x);
                    max_x = max_x.max(corner.x);
                    min_y = min_y.min(corner.y);
                    max_y = max_y.max(corner.y);
                }

                delta.x = delta.x.clamp(-min_x, world_max_x.max(1.0) - max_x);
                delta.y = delta.y.clamp(-min_y, world_max_y.max(1.0) - max_y);

                surface.corners = editor
                    .drag_start_corners
                    .map(|corner| vec2(corner.x + delta.x, corner.y + delta.y));
                changed = true;
            }
            SurfaceDragMode::Rotate => {
                let center_screen = world_to_screen(
                    editor.rotate_center_world,
                    world_max_x,
                    world_max_y,
                    left,
                    right,
                    top,
                    bottom,
                );
                let current_angle_rad =
                    (mouse_screen.y - center_screen.y).atan2(mouse_screen.x - center_screen.x);
                let delta_angle_rad = -(current_angle_rad - editor.rotate_start_angle_rad);

                let mut rotated = editor.drag_start_corners.map(|corner| {
                    let relative = corner - editor.rotate_center_world;
                    editor.rotate_center_world + rotate_vec(relative, delta_angle_rad)
                });

                let mut min_x = rotated[0].x;
                let mut max_x = rotated[0].x;
                let mut min_y = rotated[0].y;
                let mut max_y = rotated[0].y;
                for corner in &rotated[1..] {
                    min_x = min_x.min(corner.x);
                    max_x = max_x.max(corner.x);
                    min_y = min_y.min(corner.y);
                    max_y = max_y.max(corner.y);
                }

                let shift_x = if min_x < 0.0 {
                    -min_x
                } else if max_x > world_max_x.max(1.0) {
                    world_max_x.max(1.0) - max_x
                } else {
                    0.0
                };
                let shift_y = if min_y < 0.0 {
                    -min_y
                } else if max_y > world_max_y.max(1.0) {
                    world_max_y.max(1.0) - max_y
                } else {
                    0.0
                };

                for corner in &mut rotated {
                    corner.x = (corner.x + shift_x).max(0.0);
                    corner.y = (corner.y + shift_y).max(0.0);
                }
                surface.corners = rotated;
                changed = true;
            }
        }
    }

    hovered_surface || editor.is_dragging() || changed
}

pub(crate) fn update_launch_editor(
    config: &mut LaunchConfig,
    editor: &mut LaunchEditor,
    phase: GamePhase,
    mouse_screen: Vec2,
    launch_screen: Vec2,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> bool {
    if matches!(phase, GamePhase::Flying) {
        editor.active = false;
        editor.hovered = false;
        return false;
    }

    if !is_mouse_button_down(MouseButton::Left) {
        editor.active = false;
    }

    editor.hovered = mouse_screen.distance(launch_screen) <= (LAUNCH_HANDLE_RADIUS + 6.0);

    if is_mouse_button_pressed(MouseButton::Left) && editor.hovered {
        editor.active = true;
    }

    if !editor.active {
        return false;
    }

    let ghost_x = mouse_screen
        .x
        .min(launch_screen.x - LAUNCH_DRAG_MIN_PX)
        .clamp(0.0, launch_screen.x - LAUNCH_DRAG_MIN_PX);
    let ghost_y = mouse_screen
        .y
        .clamp(top, bottom + LAUNCH_GHOST_BELOW_AXIS_PX);
    editor.ghost_screen = vec2(ghost_x, ghost_y);

    let px_per_world_x = ((right - left) / world_max_x.max(1.0)).max(1e-6);
    let px_per_world_y = ((bottom - top) / world_max_y.max(1.0)).max(1e-6);
    let launch_vx = ((launch_screen.x - editor.ghost_screen.x) / px_per_world_x).max(0.0);
    let launch_vy = if SLINGSHOT_VERTICAL_MIRROR {
        (editor.ghost_screen.y - launch_screen.y) / px_per_world_y
    } else {
        (launch_screen.y - editor.ghost_screen.y) / px_per_world_y
    };
    let speed = (launch_vx.powi(2) + launch_vy.powi(2))
        .sqrt()
        .clamp(5.0, 500.0);
    let angle = launch_vy
        .atan2(launch_vx.max(1e-6))
        .to_degrees()
        .clamp(-89.0, 89.0);

    config.speed_mps = speed;
    config.angle_deg = angle;
    true
}
