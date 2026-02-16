use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

const INITIAL_WINDOW_WIDTH: i32 = 1920;
const INITIAL_WINDOW_HEIGHT: i32 = 1080;
const MSAA_SAMPLES: i32 = 4;
const UI_FONT_PATH: &str = "assets/fonts/Lato-Regular.ttf";

const LEFT_MARGIN: f32 = 60.0;
const RIGHT_MARGIN: f32 = 30.0;
const TOP_MARGIN: f32 = 140.0;
const BOTTOM_MARGIN: f32 = 130.0;

const TITLE_Y: f32 = 46.0;
const CONTROLS_Y: f32 = 92.0;
const TRAJECTORY_SAMPLES: usize = 320;
const FIXED_STEP_S: f32 = 1.0 / 240.0;
const MAX_SIM_TIME_S: f32 = 60.0;
const DISTANCE_TO_HEIGHT_RATIO: f32 = 2.0; // x:y data window ratio
const X_GRID_LINES: usize = 10;
const Y_GRID_LINES: usize = 8;

#[derive(Clone, Copy)]
struct LaunchConfig {
    angle_deg: f32,
    speed_mps: f32,
    height_m: f32,
}

#[derive(Clone, Copy)]
struct Environment {
    name: &'static str,
    gravity_mps2: f32,
    wind_accel_x_mps2: f32,
    drag_linear: f32,
}

#[derive(Clone, Copy)]
struct BounceSurface {
    x_start: f32,
    x_end: f32,
    y: f32,
    restitution: f32,
}

#[derive(Clone, Copy)]
struct Target {
    center: Vec2,
    radius_m: f32,
}

#[derive(Clone, Copy)]
struct Barrier {
    rect: Rect,
}

struct Level {
    code: &'static str,
    title: &'static str,
    environment: Environment,
    target: Target,
    bounce_surface: BounceSurface,
    barriers: Vec<Barrier>,
    required_bounces: u32,
}

impl Level {
    fn moon_level_two() -> Self {
        Self {
            code: "MOON 2",
            title: "Bounce Into Target",
            environment: Environment {
                name: "Moon",
                gravity_mps2: 1.62,
                wind_accel_x_mps2: 0.0,
                drag_linear: 0.0,
            },
            target: Target {
                center: vec2(980.0, 190.0),
                radius_m: 35.0,
            },
            bounce_surface: BounceSurface {
                x_start: 380.0,
                x_end: 690.0,
                y: 85.0,
                restitution: 0.9,
            },
            barriers: vec![],
            required_bounces: 1,
        }
    }
}

#[derive(Clone, Copy)]
struct Projectile {
    position: Vec2,
    velocity: Vec2,
    elapsed_s: f32,
    bounces: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepOutcome {
    Flying,
    HitTarget,
    HitGround,
    HitBarrier,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GamePhase {
    Aiming,
    Flying,
    Success,
    Failed,
}

struct GameState {
    phase: GamePhase,
    shot: Option<Projectile>,
    trail: Vec<Vec2>,
    paused: bool,
    status_line: String,
}

impl GameState {
    fn new() -> Self {
        Self {
            phase: GamePhase::Aiming,
            shot: None,
            trail: Vec::new(),
            paused: false,
            status_line: "Ready".to_string(),
        }
    }

    fn launch(&mut self, config: LaunchConfig) {
        self.phase = GamePhase::Flying;
        self.paused = false;
        self.shot = Some(launch_projectile(config));
        self.trail.clear();
        self.trail.push(vec2(0.0, config.height_m.max(0.0)));
        self.status_line = "Shot launched".to_string();
    }

    fn reset(&mut self) {
        self.phase = GamePhase::Aiming;
        self.shot = None;
        self.trail.clear();
        self.paused = false;
        self.status_line = "Reset".to_string();
    }
}

struct Prediction {
    points: Vec<Vec2>,
    range_m: f32,
    flight_time_s: f32,
    bounces: u32,
    outcome: StepOutcome,
}

fn launch_velocity(config: LaunchConfig) -> Vec2 {
    let theta = config.angle_deg.to_radians();
    vec2(
        config.speed_mps * theta.cos(),
        config.speed_mps * theta.sin(),
    )
}

fn launch_projectile(config: LaunchConfig) -> Projectile {
    Projectile {
        position: vec2(0.0, config.height_m.max(0.0)),
        velocity: launch_velocity(config),
        elapsed_s: 0.0,
        bounces: 0,
    }
}

fn step_projectile(projectile: &mut Projectile, level: &Level, dt: f32) -> StepOutcome {
    let env = level.environment;
    let prev = projectile.position;

    // Wind + linear drag hooks are in place even though Moon level uses zeros.
    let ax = env.wind_accel_x_mps2 - (env.drag_linear * projectile.velocity.x);
    let ay = -env.gravity_mps2 - (env.drag_linear * projectile.velocity.y);
    projectile.velocity.x += ax * dt;
    projectile.velocity.y += ay * dt;
    projectile.position += projectile.velocity * dt;
    projectile.elapsed_s += dt;

    let surface = level.bounce_surface;
    let crossed_surface = prev.y >= surface.y && projectile.position.y <= surface.y;
    let on_surface_x =
        projectile.position.x >= surface.x_start && projectile.position.x <= surface.x_end;
    if crossed_surface && on_surface_x && projectile.velocity.y < 0.0 {
        projectile.position.y = surface.y + 0.01;
        projectile.velocity.y = -projectile.velocity.y * surface.restitution;
        projectile.velocity.x *= 0.985;
        projectile.bounces += 1;
    }

    if level
        .barriers
        .iter()
        .any(|barrier| barrier.rect.contains(projectile.position))
    {
        return StepOutcome::HitBarrier;
    }

    if projectile.position.distance(level.target.center) <= level.target.radius_m
        && projectile.bounces >= level.required_bounces
    {
        return StepOutcome::HitTarget;
    }

    if projectile.position.y <= 0.0 {
        projectile.position.y = 0.0;
        return StepOutcome::HitGround;
    }

    StepOutcome::Flying
}

fn simulate_prediction(config: LaunchConfig, level: &Level) -> Prediction {
    let mut projectile = launch_projectile(config);
    let mut points = vec![projectile.position];
    let mut outcome = StepOutcome::Flying;

    for _ in 0..(TRAJECTORY_SAMPLES * 6) {
        outcome = step_projectile(&mut projectile, level, FIXED_STEP_S);
        points.push(projectile.position);
        if outcome != StepOutcome::Flying || projectile.elapsed_s > MAX_SIM_TIME_S {
            break;
        }
    }

    Prediction {
        points,
        range_m: projectile.position.x.max(0.0),
        flight_time_s: projectile.elapsed_s,
        bounces: projectile.bounces,
        outcome,
    }
}

fn axis_window(raw_max_x: f32, raw_max_y: f32) -> (f32, f32) {
    let raw_x_span = raw_max_x.max(1.0);
    let raw_y_span = raw_max_y.max(1.0);
    let x_pad = raw_x_span * 0.06;
    let y_pad = raw_y_span * 0.10;

    let mut x_span = (raw_max_x + x_pad).max(1.0);
    let mut y_span = (raw_max_y + y_pad).max(1.0);

    if x_span / y_span < DISTANCE_TO_HEIGHT_RATIO {
        x_span = y_span * DISTANCE_TO_HEIGHT_RATIO;
    } else {
        y_span = x_span / DISTANCE_TO_HEIGHT_RATIO;
    }

    (x_span, y_span)
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

fn format_axis_value(value: f32, axis_max: f32) -> String {
    if axis_max >= 1000.0 {
        format!("{value:.0}")
    } else if axis_max >= 100.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

fn draw_ui_text(text: &str, x: f32, y: f32, font_size: u16, color: Color, font: Option<&Font>) {
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

fn draw_grid(left: f32, right: f32, top: f32, bottom: f32, color: Color) {
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

fn draw_axis_tick_labels(
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

fn draw_level_objects(
    level: &Level,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) {
    let surface = level.bounce_surface;
    let s0 = world_to_screen(
        vec2(surface.x_start, surface.y),
        world_max_x,
        world_max_y,
        left,
        right,
        top,
        bottom,
    );
    let s1 = world_to_screen(
        vec2(surface.x_end, surface.y),
        world_max_x,
        world_max_y,
        left,
        right,
        top,
        bottom,
    );
    draw_line(
        s0.x,
        s0.y,
        s1.x,
        s1.y,
        7.0,
        Color::from_rgba(242, 159, 5, 255),
    );

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

fn draw_path(
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

fn window_conf() -> Conf {
    Conf {
        window_title: "ParabolicRust Interactive".to_string(),
        window_width: INITIAL_WINDOW_WIDTH,
        window_height: INITIAL_WINDOW_HEIGHT,
        high_dpi: true,
        sample_count: MSAA_SAMPLES,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let ui_font = match load_ttf_font(UI_FONT_PATH).await {
        Ok(font) => Some(font),
        Err(err) => {
            println!("Could not load '{UI_FONT_PATH}': {err}. Falling back to default font.");
            None
        }
    };

    let level = Level::moon_level_two();
    let mut config = LaunchConfig {
        angle_deg: 28.0,
        speed_mps: 145.0,
        height_m: 22.0,
    };
    let mut game = GameState::new();
    let mut show_preview = true;

    loop {
        let frame_dt = get_frame_time();
        let screen_w = screen_width();
        let screen_h = screen_height();

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

        // UI panel with sliders/buttons for "real" controls.
        let mut launch_clicked = false;
        let mut reset_clicked = false;
        widgets::Window::new(hash!(), vec2(18.0, 120.0), vec2(360.0, 300.0))
            .label("Moon Level 2 Controls")
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, &format!("Environment: {}", level.environment.name));
                ui.label(
                    None,
                    &format!(
                        "g = {:.2} m/s^2 | wind = {:.2} | drag = {:.2}",
                        level.environment.gravity_mps2,
                        level.environment.wind_accel_x_mps2,
                        level.environment.drag_linear
                    ),
                );
                ui.separator();
                ui.slider(hash!(), "Angle (deg)", -89.0..89.0, &mut config.angle_deg);
                ui.slider(hash!(), "Velocity (m/s)", 5.0..500.0, &mut config.speed_mps);
                ui.slider(hash!(), "Height (m)", 0.0..400.0, &mut config.height_m);
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

        let prediction = simulate_prediction(config, &level);

        if matches!(game.phase, GamePhase::Flying) && !game.paused {
            let mut remaining = frame_dt.min(0.05);
            while remaining > 0.0 {
                let dt = remaining.min(FIXED_STEP_S);
                remaining -= dt;

                if let Some(shot) = game.shot.as_mut() {
                    let outcome = step_projectile(shot, &level, dt);
                    game.trail.push(shot.position);

                    if outcome != StepOutcome::Flying {
                        game.phase = if outcome == StepOutcome::HitTarget {
                            GamePhase::Success
                        } else {
                            GamePhase::Failed
                        };
                        game.status_line = match outcome {
                            StepOutcome::HitTarget => {
                                format!(
                                    "Target hit in {:.2}s with {} bounce(s)",
                                    shot.elapsed_s, shot.bounces
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

        let mut raw_max_x = prediction
            .range_m
            .max(level.target.center.x + level.target.radius_m)
            .max(level.bounce_surface.x_end)
            .max(1.0);
        let mut raw_max_y = prediction
            .points
            .iter()
            .fold(0.0f32, |acc, p| acc.max(p.y))
            .max(config.height_m)
            .max(level.target.center.y + level.target.radius_m)
            .max(level.bounce_surface.y)
            .max(1.0);
        if let Some(shot) = game.shot {
            raw_max_x = raw_max_x.max(shot.position.x);
            raw_max_y = raw_max_y.max(shot.position.y);
        }
        let (world_max_x, world_max_y) = axis_window(raw_max_x, raw_max_y);

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
        draw_level_objects(&level, world_max_x, world_max_y, left, right, top, bottom);

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

        // Keep your duplicated range label near x-axis intersection.
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

        // Header + status lines
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
                level.code, level.title, level.environment.name
            ),
            left,
            TITLE_Y + 30.0,
            22,
            DARKGRAY,
            ui_font.as_ref(),
        );
        draw_ui_text(
            "Controls: Space launch/pause | R reset | Use sliders in panel",
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
                "Flight: {:.2} s | Range: {:.2} m | Bounces: {} | State: {}",
                active_time, active_range, active_bounces, phase_text
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
