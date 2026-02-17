use macroquad::prelude::*;
use parabolic_rust::core::window::fixed_ratio_axis_window_f32;

use crate::constants::{FIXED_STEP_S, MAX_SIM_TIME_S, TRAJECTORY_SAMPLES};
use crate::model::{BounceSurface, LaunchConfig, Level, Prediction, Projectile, StepOutcome};

pub(crate) fn launch_velocity(config: LaunchConfig) -> Vec2 {
    let theta = config.angle_deg.to_radians();
    vec2(
        config.speed_mps * theta.cos(),
        config.speed_mps * theta.sin(),
    )
}

pub(crate) fn launch_projectile(config: LaunchConfig) -> Projectile {
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

pub(crate) fn segment_intersection(p: Vec2, p2: Vec2, q: Vec2, q2: Vec2) -> Option<(f32, f32)> {
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

pub(crate) fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
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

pub(crate) fn bounce_surface_edges(corners: &[Vec2; 4]) -> [(Vec2, Vec2); 4] {
    [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ]
}

pub(crate) fn quad_center(corners: &[Vec2; 4]) -> Vec2 {
    (corners[0] + corners[1] + corners[2] + corners[3]) * 0.25
}

pub(crate) fn rotate_vec(v: Vec2, angle_rad: f32) -> Vec2 {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    vec2((v.x * cos_a) - (v.y * sin_a), (v.x * sin_a) + (v.y * cos_a))
}

pub(crate) fn rotation_handle_screen(corners_screen: &[Vec2; 4]) -> (Vec2, Vec2) {
    let edge_mid = (corners_screen[0] + corners_screen[1]) * 0.5;
    let center = quad_center(corners_screen);
    let edge_dir = (corners_screen[1] - corners_screen[0]).normalize_or_zero();

    if edge_dir.length_squared() < 1e-8 {
        return (
            edge_mid,
            edge_mid + vec2(0.0, -crate::constants::ROTATE_HANDLE_STICK_PX),
        );
    }

    let mut outward = vec2(-edge_dir.y, edge_dir.x).normalize_or_zero();
    if outward.dot(center - edge_mid) > 0.0 {
        outward = -outward;
    }
    (
        edge_mid,
        edge_mid + (outward * crate::constants::ROTATE_HANDLE_STICK_PX),
    )
}

pub(crate) fn step_projectile(projectile: &mut Projectile, level: &Level, dt: f32) -> StepOutcome {
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
        resolve_surface_bounce(projectile, surface, prev);
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

fn resolve_surface_bounce(projectile: &mut Projectile, surface: BounceSurface, prev: Vec2) {
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

pub(crate) fn simulate_prediction(config: LaunchConfig, level: &Level) -> Prediction {
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
    fixed_ratio_axis_window_f32(raw_max_x, raw_max_y)
}

pub(crate) fn world_to_screen(
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

pub(crate) fn screen_to_world(
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

pub(crate) fn compute_world_window(
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
