use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

const INITIAL_WINDOW_WIDTH: i32 = 1920;
const INITIAL_WINDOW_HEIGHT: i32 = 1080;
const MSAA_SAMPLES: i32 = 4;
const UI_FONT_PATH: &str = "assets/fonts/Lato-Regular.ttf";

const LEFT_MARGIN: f32 = 120.0;
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
const TITLE_SCREEN_BG: Color = Color::new(0.92, 0.93, 0.95, 1.0);
const START_BUTTON_COLOR: Color = Color::new(0.14, 0.45, 0.95, 1.0);
const START_BUTTON_TEXT: &str = "Start Game";
const SURFACE_HANDLE_RADIUS: f32 = 8.0;
const ROTATE_HANDLE_RADIUS: f32 = 9.0;
const ROTATE_HANDLE_STICK_PX: f32 = 34.0;
const LAUNCH_HANDLE_RADIUS: f32 = 8.0;
const LAUNCH_GHOST_RADIUS: f32 = 7.0;
const LAUNCH_DRAG_MIN_PX: f32 = 10.0;
const LAUNCH_GHOST_BELOW_AXIS_PX: f32 = 220.0;
const HEIGHT_KEY_RATE_MPS: f32 = 90.0;
const VELOCITY_KEY_RATE_MPS: f32 = 140.0;
const SLINGSHOT_VERTICAL_MIRROR: bool = true;

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
    corners: [Vec2; 4],
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
    bounce_surface: Option<BounceSurface>,
    barriers: Vec<Barrier>,
    required_bounces: u32,
    default_launch: LaunchConfig,
}

impl Level {
    fn moon_campaign() -> Vec<Self> {
        let moon_env = Environment {
            name: "Moon",
            gravity_mps2: 1.62,
            wind_accel_x_mps2: 0.0,
            drag_linear: 0.0,
        };

        vec![
            Self {
                code: "MOON 1",
                title: "Direct Shot",
                environment: moon_env,
                target: Target {
                    center: vec2(700.0, 130.0),
                    radius_m: 30.0,
                },
                bounce_surface: None,
                barriers: vec![],
                required_bounces: 0,
                default_launch: LaunchConfig {
                    angle_deg: 18.0,
                    speed_mps: 90.0,
                    height_m: 20.0,
                },
            },
            Self {
                code: "MOON 2",
                title: "Bounce Into Target",
                environment: moon_env,
                target: Target {
                    center: vec2(980.0, 190.0),
                    radius_m: 35.0,
                },
                bounce_surface: Some(BounceSurface {
                    corners: [
                        vec2(380.0, 95.0),
                        vec2(690.0, 95.0),
                        vec2(690.0, 75.0),
                        vec2(380.0, 75.0),
                    ],
                    restitution: 0.9,
                }),
                barriers: vec![],
                required_bounces: 1,
                default_launch: LaunchConfig {
                    angle_deg: 28.0,
                    speed_mps: 145.0,
                    height_m: 22.0,
                },
            },
            Self {
                code: "MOON 3",
                title: "Thread The Gap",
                environment: moon_env,
                target: Target {
                    center: vec2(980.0, 190.0),
                    radius_m: 32.0,
                },
                bounce_surface: None,
                barriers: vec![
                    Barrier {
                        rect: Rect::new(560.0, 0.0, 38.0, 230.0),
                    },
                    Barrier {
                        rect: Rect::new(560.0, 310.0, 38.0, 260.0),
                    },
                ],
                required_bounces: 0,
                default_launch: LaunchConfig {
                    angle_deg: 20.0,
                    speed_mps: 145.0,
                    height_m: 24.0,
                },
            },
            Self {
                code: "MOON 4",
                title: "Bank Shot Through Gap",
                environment: moon_env,
                target: Target {
                    center: vec2(1110.0, 220.0),
                    radius_m: 32.0,
                },
                bounce_surface: Some(BounceSurface {
                    corners: [
                        vec2(420.0, 106.0),
                        vec2(710.0, 106.0),
                        vec2(710.0, 84.0),
                        vec2(420.0, 84.0),
                    ],
                    restitution: 0.88,
                }),
                barriers: vec![
                    Barrier {
                        rect: Rect::new(790.0, 0.0, 36.0, 250.0),
                    },
                    Barrier {
                        rect: Rect::new(790.0, 340.0, 36.0, 260.0),
                    },
                ],
                required_bounces: 1,
                default_launch: LaunchConfig {
                    angle_deg: 24.0,
                    speed_mps: 170.0,
                    height_m: 30.0,
                },
            },
        ]
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppScene {
    Title,
    Game,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum SurfaceDragMode {
    Corner(usize),
    Surface,
    Rotate,
}

struct SurfaceEditor {
    drag_mode: Option<SurfaceDragMode>,
    hovered_corner: Option<usize>,
    hovered_surface: bool,
    hovered_rotate: bool,
    drag_start_mouse_world: Vec2,
    drag_start_corners: [Vec2; 4],
    rotate_center_world: Vec2,
    rotate_start_angle_rad: f32,
}

impl SurfaceEditor {
    fn new() -> Self {
        Self {
            drag_mode: None,
            hovered_corner: None,
            hovered_surface: false,
            hovered_rotate: false,
            drag_start_mouse_world: Vec2::ZERO,
            drag_start_corners: [Vec2::ZERO; 4],
            rotate_center_world: Vec2::ZERO,
            rotate_start_angle_rad: 0.0,
        }
    }

    fn is_dragging(&self) -> bool {
        self.drag_mode.is_some()
    }

    fn active_corner(&self) -> Option<usize> {
        match self.drag_mode {
            Some(SurfaceDragMode::Corner(idx)) => Some(idx),
            _ => None,
        }
    }

    fn active_rotate(&self) -> bool {
        matches!(self.drag_mode, Some(SurfaceDragMode::Rotate))
    }
}

struct LaunchEditor {
    active: bool,
    hovered: bool,
    ghost_screen: Vec2,
}

impl LaunchEditor {
    fn new() -> Self {
        Self {
            active: false,
            hovered: false,
            ghost_screen: Vec2::ZERO,
        }
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

fn cross_2d(a: Vec2, b: Vec2) -> f32 {
    (a.x * b.y) - (a.y * b.x)
}

fn segment_intersection(p: Vec2, p2: Vec2, q: Vec2, q2: Vec2) -> Option<(f32, f32)> {
    let r = p2 - p;
    let s = q2 - q;
    let rxs = cross_2d(r, s);
    let qmp = q - p;

    if rxs.abs() < 1e-6 {
        return None;
    }

    let t = cross_2d(qmp, s) / rxs;
    let u = cross_2d(qmp, r) / rxs;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some((t, u))
    } else {
        None
    }
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];
        let intersect = ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y + 1e-6) + pi.x);
        if intersect {
            inside = !inside;
        }
        j = i;
    }

    inside
}

fn bounce_surface_edges(corners: &[Vec2; 4]) -> [(Vec2, Vec2); 4] {
    [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ]
}

fn quad_center(corners: &[Vec2; 4]) -> Vec2 {
    (corners[0] + corners[1] + corners[2] + corners[3]) * 0.25
}

fn rotate_vec(v: Vec2, angle_rad: f32) -> Vec2 {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    vec2((v.x * cos_a) - (v.y * sin_a), (v.x * sin_a) + (v.y * cos_a))
}

fn rotation_handle_screen(corners_screen: &[Vec2; 4]) -> (Vec2, Vec2) {
    let edge_mid = (corners_screen[0] + corners_screen[1]) * 0.5;
    let center = quad_center(corners_screen);
    let edge_dir = (corners_screen[1] - corners_screen[0]).normalize_or_zero();

    if edge_dir.length_squared() < 1e-8 {
        return (edge_mid, edge_mid + vec2(0.0, -ROTATE_HANDLE_STICK_PX));
    }

    let mut outward = vec2(-edge_dir.y, edge_dir.x).normalize_or_zero();
    if outward.dot(center - edge_mid) > 0.0 {
        outward = -outward;
    }
    (edge_mid, edge_mid + (outward * ROTATE_HANDLE_STICK_PX))
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

    if let Some(surface) = level.bounce_surface {
        let edges = bounce_surface_edges(&surface.corners);
        let mut best_hit: Option<(f32, Vec2, Vec2)> = None;

        for (a, b) in edges {
            if let Some((t, _u)) = segment_intersection(prev, projectile.position, a, b) {
                let intersection = prev + ((projectile.position - prev) * t);
                let edge_dir = (b - a).normalize_or_zero();
                if edge_dir.length_squared() < 1e-8 {
                    continue;
                }
                let mut normal = vec2(-edge_dir.y, edge_dir.x).normalize_or_zero();
                if normal.length_squared() < 1e-8 {
                    continue;
                }
                if projectile.velocity.dot(normal) > 0.0 {
                    normal = -normal;
                }

                if best_hit.is_none_or(|(best_t, _, _)| t < best_t) {
                    best_hit = Some((t, intersection, normal));
                }
            }
        }

        if let Some((_t, intersection, normal)) = best_hit {
            let vn = projectile.velocity.dot(normal);
            projectile.velocity -= normal * ((1.0 + surface.restitution) * vn);
            projectile.velocity *= 0.995;
            projectile.position = intersection + (normal * 0.05);
            projectile.bounces += 1;
        } else if point_in_polygon(projectile.position, &surface.corners) {
            // Fallback if step ends inside surface without a clean edge intersection.
            projectile.velocity.y = projectile.velocity.y.abs() * surface.restitution;
            projectile.position.y += 0.05;
            projectile.bounces += 1;
        }
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

fn screen_to_world(
    screen: Vec2,
    world_max_x: f32,
    world_max_y: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> Vec2 {
    let plot_w = (right - left).max(1.0);
    let plot_h = (bottom - top).max(1.0);
    let world_x = ((screen.x - left) / plot_w) * world_max_x.max(1.0);
    let world_y = ((bottom - screen.y) / plot_h) * world_max_y.max(1.0);
    vec2(world_x.max(0.0), world_y.max(0.0))
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

fn compute_world_window(
    level: &Level,
    config: LaunchConfig,
    prediction: &Prediction,
    shot: Option<Projectile>,
) -> (f32, f32) {
    let mut raw_max_x = prediction
        .range_m
        .max(level.target.center.x + level.target.radius_m)
        .max(1.0);
    let mut raw_max_y = prediction
        .points
        .iter()
        .fold(0.0f32, |acc, p| acc.max(p.y))
        .max(config.height_m)
        .max(level.target.center.y + level.target.radius_m)
        .max(1.0);

    if let Some(surface) = level.bounce_surface {
        for corner in surface.corners {
            raw_max_x = raw_max_x.max(corner.x);
            raw_max_y = raw_max_y.max(corner.y);
        }
    }

    for barrier in &level.barriers {
        raw_max_x = raw_max_x.max(barrier.rect.x + barrier.rect.w);
        raw_max_y = raw_max_y.max(barrier.rect.y + barrier.rect.h);
    }

    if let Some(shot) = shot {
        raw_max_x = raw_max_x.max(shot.position.x);
        raw_max_y = raw_max_y.max(shot.position.y);
    }

    axis_window(raw_max_x, raw_max_y)
}

fn update_surface_editor(
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

fn draw_level_objects(
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

fn update_launch_editor(
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

fn draw_launch_editor(launch_screen: Vec2, editor: &LaunchEditor) {
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

fn draw_title_screen(screen_w: f32, screen_h: f32, font: Option<&Font>) -> bool {
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
