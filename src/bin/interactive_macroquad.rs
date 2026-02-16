use macroquad::prelude::*;

const G: f32 = 9.8;
const LEFT_MARGIN: f32 = 60.0;
const RIGHT_MARGIN: f32 = 30.0;
const TOP_MARGIN: f32 = 30.0;
const BOTTOM_MARGIN: f32 = 70.0;
const TRAJECTORY_SAMPLES: usize = 300;

#[derive(Clone, Copy)]
struct Inputs {
    angle_deg: f32,
    speed_mps: f32,
    height_m: f32,
}

fn velocity_components(inputs: Inputs) -> (f32, f32) {
    let theta = inputs.angle_deg.to_radians();
    let vx = inputs.speed_mps * theta.cos();
    let vy = inputs.speed_mps * theta.sin();
    (vx, vy)
}

fn flight_time_s(inputs: Inputs) -> f32 {
    let (_, vy) = velocity_components(inputs);
    let disc = vy * vy + (2.0 * G * inputs.height_m);
    if disc <= 0.0 {
        0.0
    } else {
        ((vy + disc.sqrt()) / G).max(0.0)
    }
}

fn trajectory_at_time(inputs: Inputs, t: f32) -> Vec2 {
    let (vx, vy) = velocity_components(inputs);
    let x = vx * t;
    let y = inputs.height_m + (vy * t) - (0.5 * G * t * t);
    vec2(x, y.max(0.0))
}

fn range_m(inputs: Inputs) -> f32 {
    let t = flight_time_s(inputs);
    trajectory_at_time(inputs, t).x.max(0.0)
}

fn apex_height_m(inputs: Inputs) -> f32 {
    let (_, vy) = velocity_components(inputs);
    if vy <= 0.0 {
        inputs.height_m.max(0.0)
    } else {
        (inputs.height_m + (vy * vy) / (2.0 * G)).max(0.0)
    }
}

fn world_to_screen(
    world: Vec2,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> Vec2 {
    let plot_w = (right - left).max(1.0);
    let plot_h = (bottom - top).max(1.0);
    let x = left + (world.x / world_max_x.max(1.0)) * plot_w;
    let y = bottom - (world.y / world_max_y.max(1.0)) * plot_h;
    vec2(x, y)
}

fn draw_grid(left: f32, right: f32, top: f32, bottom: f32, color: Color) {
    let x_lines = 10;
    let y_lines = 8;

    for i in 0..=x_lines {
        let t = i as f32 / x_lines as f32;
        let x = left + t * (right - left);
        draw_line(x, top, x, bottom, 1.0, color);
    }
    for i in 0..=y_lines {
        let t = i as f32 / y_lines as f32;
        let y = bottom - t * (bottom - top);
        draw_line(left, y, right, y, 1.0, color);
    }
}

#[macroquad::main("ParabolicRust Interactive")]
async fn main() {
    let mut inputs = Inputs {
        angle_deg: 45.0,
        speed_mps: 60.0,
        height_m: 20.0,
    };
    let mut t_now = 0.0f32;
    let mut is_playing = false;
    let mut sim_speed = 1.0f32;
    let mut previous_inputs = inputs;

    loop {
        let dt = get_frame_time();
        let screen_w = screen_width();
        let screen_h = screen_height();
        let left = LEFT_MARGIN;
        let right = screen_w - RIGHT_MARGIN;
        let top = TOP_MARGIN;
        let bottom = screen_h - BOTTOM_MARGIN;

        if is_key_pressed(KeyCode::Space) {
            is_playing = !is_playing;
        }
        if is_key_pressed(KeyCode::R) {
            t_now = 0.0;
            is_playing = false;
        }
        if is_key_pressed(KeyCode::Up) {
            sim_speed = (sim_speed + 0.25).min(8.0);
        }
        if is_key_pressed(KeyCode::Down) {
            sim_speed = (sim_speed - 0.25).max(0.25);
        }

        if is_key_down(KeyCode::Q) {
            inputs.angle_deg = (inputs.angle_deg + 35.0 * dt).min(89.0);
        }
        if is_key_down(KeyCode::A) {
            inputs.angle_deg = (inputs.angle_deg - 35.0 * dt).max(-89.0);
        }
        if is_key_down(KeyCode::W) {
            inputs.speed_mps = (inputs.speed_mps + 40.0 * dt).min(3000.0);
        }
        if is_key_down(KeyCode::S) {
            inputs.speed_mps = (inputs.speed_mps - 40.0 * dt).max(0.0);
        }
        if is_key_down(KeyCode::E) {
            inputs.height_m = (inputs.height_m + 60.0 * dt).min(5000.0);
        }
        if is_key_down(KeyCode::D) {
            inputs.height_m = (inputs.height_m - 60.0 * dt).max(0.0);
        }

        if (inputs.angle_deg - previous_inputs.angle_deg).abs() > f32::EPSILON
            || (inputs.speed_mps - previous_inputs.speed_mps).abs() > f32::EPSILON
            || (inputs.height_m - previous_inputs.height_m).abs() > f32::EPSILON
        {
            t_now = 0.0;
            is_playing = false;
            previous_inputs = inputs;
        }

        let flight_time = flight_time_s(inputs);
        let horizontal_range = range_m(inputs);
        let max_y = apex_height_m(inputs).max(inputs.height_m).max(1.0) * 1.08;
        let max_x = horizontal_range.max(1.0) * 1.08;

        if is_playing {
            t_now += dt * sim_speed;
            if t_now >= flight_time {
                t_now = flight_time;
                is_playing = false;
            }
        } else if t_now > flight_time {
            t_now = flight_time;
        }

        clear_background(Color::from_rgba(250, 251, 253, 255));
        draw_grid(
            left,
            right,
            top,
            bottom,
            Color::from_rgba(227, 231, 236, 255),
        );

        // Axes
        draw_line(left, bottom, right, bottom, 2.0, DARKGRAY); // x-axis (ground)
        draw_line(left, top, left, bottom, 2.0, DARKGRAY); // y-axis

        let launch_px = world_to_screen(
            vec2(0.0, inputs.height_m),
            max_x,
            max_y,
            left,
            right,
            top,
            bottom,
        );
        let landing_px = world_to_screen(
            vec2(horizontal_range, 0.0),
            max_x,
            max_y,
            left,
            right,
            top,
            bottom,
        );

        // Draw trajectory
        let mut previous = world_to_screen(
            vec2(0.0, inputs.height_m),
            max_x,
            max_y,
            left,
            right,
            top,
            bottom,
        );
        for i in 1..=TRAJECTORY_SAMPLES {
            let t = flight_time * (i as f32 / TRAJECTORY_SAMPLES as f32);
            let world = trajectory_at_time(inputs, t);
            let current = world_to_screen(world, max_x, max_y, left, right, top, bottom);
            draw_line(
                previous.x,
                previous.y,
                current.x,
                current.y,
                3.0,
                Color::from_rgba(54, 123, 245, 255),
            );
            previous = current;
        }

        // Animated projectile
        let projectile_world = trajectory_at_time(inputs, t_now);
        let projectile_px =
            world_to_screen(projectile_world, max_x, max_y, left, right, top, bottom);
        draw_circle(projectile_px.x, projectile_px.y, 7.0, RED);
        draw_circle_lines(projectile_px.x, projectile_px.y, 7.0, 2.0, MAROON);

        // Markers
        draw_circle(launch_px.x, launch_px.y, 5.0, ORANGE);
        draw_circle(landing_px.x, landing_px.y, 5.0, GREEN);

        // HUD
        let status = if is_playing { "Playing" } else { "Paused" };
        let label_color = Color::from_rgba(30, 30, 35, 255);
        draw_text(
            "ParabolicRust Interactive (macroquad)",
            left,
            24.0,
            28.0,
            label_color,
        );
        draw_text(
            &format!(
                "Angle: {:.1} deg | Velocity: {:.1} m/s | Height: {:.1} m",
                inputs.angle_deg, inputs.speed_mps, inputs.height_m
            ),
            left,
            screen_h - 40.0,
            24.0,
            label_color,
        );
        draw_text(
            &format!(
                "Flight: {:.2} s | Range: {:.2} m | t: {:.2} s | Speed x{:.2} | {}",
                flight_time, horizontal_range, t_now, sim_speed, status
            ),
            left,
            screen_h - 14.0,
            22.0,
            label_color,
        );

        draw_text(
            "Controls: Space play/pause | R reset | Q/A angle | W/S velocity | E/D height | Up/Down sim speed",
            left,
            top + 18.0,
            20.0,
            DARKGRAY,
        );

        next_frame().await;
    }
}
