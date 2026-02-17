use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::physics::launch_projectile;

#[derive(Clone, Copy)]
pub(crate) struct LaunchConfig {
    pub(crate) angle_deg: f32,
    pub(crate) speed_mps: f32,
    pub(crate) height_m: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Environment {
    pub(crate) name: &'static str,
    pub(crate) gravity_mps2: f32,
    pub(crate) wind_accel_x_mps2: f32,
    pub(crate) drag_linear: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct BounceSurface {
    pub(crate) corners: [Vec2; 4],
    pub(crate) restitution: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Target {
    pub(crate) center: Vec2,
    pub(crate) radius_m: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Barrier {
    pub(crate) rect: Rect,
}

pub(crate) struct Level {
    pub(crate) code: &'static str,
    pub(crate) title: &'static str,
    pub(crate) level_in_environment: usize,
    pub(crate) environment: Environment,
    pub(crate) target: Target,
    pub(crate) bounce_surface: Option<BounceSurface>,
    pub(crate) barriers: Vec<Barrier>,
    pub(crate) required_bounces: u32,
    pub(crate) default_launch: LaunchConfig,
}

impl Level {
    pub(crate) fn campaign() -> Vec<Self> {
        let mut levels = Self::earth_campaign();
        levels.extend(Self::moon_campaign());
        levels
    }

    fn random_earth_wind_mps2() -> f32 {
        let magnitude = gen_range(0.20f32, 0.80f32);
        let sign = if gen_range(0i32, 2i32) == 0 {
            -1.0
        } else {
            1.0
        };
        sign * magnitude
    }

    pub(crate) fn earth_campaign() -> Vec<Self> {
        let earth_drag = 0.015;
        vec![
            Self {
                code: "EARTH 1",
                title: "Direct Shot",
                level_in_environment: 1,
                environment: Environment {
                    name: "Earth",
                    gravity_mps2: 9.8,
                    wind_accel_x_mps2: Self::random_earth_wind_mps2(),
                    drag_linear: earth_drag,
                },
                target: Target {
                    center: vec2(165.0, 28.0),
                    radius_m: 12.0,
                },
                bounce_surface: None,
                barriers: vec![],
                required_bounces: 0,
                default_launch: LaunchConfig {
                    angle_deg: 34.0,
                    speed_mps: 56.0,
                    height_m: 2.0,
                },
            },
            Self {
                code: "EARTH 2",
                title: "Single Bounce",
                level_in_environment: 2,
                environment: Environment {
                    name: "Earth",
                    gravity_mps2: 9.8,
                    wind_accel_x_mps2: Self::random_earth_wind_mps2(),
                    drag_linear: earth_drag,
                },
                target: Target {
                    center: vec2(208.0, 36.0),
                    radius_m: 12.0,
                },
                bounce_surface: Some(BounceSurface {
                    corners: [
                        vec2(86.0, 19.0),
                        vec2(146.0, 19.0),
                        vec2(146.0, 13.0),
                        vec2(86.0, 13.0),
                    ],
                    restitution: 0.82,
                }),
                barriers: vec![],
                required_bounces: 1,
                default_launch: LaunchConfig {
                    angle_deg: 31.0,
                    speed_mps: 58.0,
                    height_m: 2.0,
                },
            },
            Self {
                code: "EARTH 3",
                title: "Thread The Gap",
                level_in_environment: 3,
                environment: Environment {
                    name: "Earth",
                    gravity_mps2: 9.8,
                    wind_accel_x_mps2: Self::random_earth_wind_mps2(),
                    drag_linear: earth_drag,
                },
                target: Target {
                    center: vec2(230.0, 34.0),
                    radius_m: 12.0,
                },
                bounce_surface: None,
                barriers: vec![
                    Barrier {
                        rect: Rect::new(133.0, 0.0, 9.0, 24.0),
                    },
                    Barrier {
                        rect: Rect::new(133.0, 58.0, 9.0, 42.0),
                    },
                ],
                required_bounces: 0,
                default_launch: LaunchConfig {
                    angle_deg: 30.0,
                    speed_mps: 64.0,
                    height_m: 2.5,
                },
            },
            Self {
                code: "EARTH 4",
                title: "Bank Shot Through Gap",
                level_in_environment: 4,
                environment: Environment {
                    name: "Earth",
                    gravity_mps2: 9.8,
                    wind_accel_x_mps2: Self::random_earth_wind_mps2(),
                    drag_linear: earth_drag,
                },
                target: Target {
                    center: vec2(280.0, 36.0),
                    radius_m: 12.0,
                },
                bounce_surface: Some(BounceSurface {
                    corners: [
                        vec2(120.0, 22.0),
                        vec2(188.0, 22.0),
                        vec2(188.0, 15.0),
                        vec2(120.0, 15.0),
                    ],
                    restitution: 0.80,
                }),
                barriers: vec![
                    Barrier {
                        rect: Rect::new(208.0, 0.0, 10.0, 28.0),
                    },
                    Barrier {
                        rect: Rect::new(208.0, 62.0, 10.0, 38.0),
                    },
                ],
                required_bounces: 1,
                default_launch: LaunchConfig {
                    angle_deg: 33.0,
                    speed_mps: 67.0,
                    height_m: 3.0,
                },
            },
        ]
    }

    pub(crate) fn moon_campaign() -> Vec<Self> {
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
                level_in_environment: 1,
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
                level_in_environment: 2,
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
                level_in_environment: 3,
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
                level_in_environment: 4,
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
pub(crate) struct Projectile {
    pub(crate) position: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) elapsed_s: f32,
    pub(crate) bounces: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepOutcome {
    Flying,
    HitTarget,
    HitGround,
    HitBarrier,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GamePhase {
    Aiming,
    Flying,
    Success,
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppScene {
    Title,
    Game,
}

pub(crate) struct GameState {
    pub(crate) phase: GamePhase,
    pub(crate) shot: Option<Projectile>,
    pub(crate) trail: Vec<Vec2>,
    pub(crate) paused: bool,
    pub(crate) status_line: String,
}

impl GameState {
    pub(crate) fn new() -> Self {
        Self {
            phase: GamePhase::Aiming,
            shot: None,
            trail: Vec::new(),
            paused: false,
            status_line: "Ready".to_string(),
        }
    }

    pub(crate) fn launch(&mut self, config: LaunchConfig) {
        self.phase = GamePhase::Flying;
        self.paused = false;
        self.shot = Some(launch_projectile(config));
        self.trail.clear();
        self.trail.push(vec2(0.0, config.height_m.max(0.0)));
        self.status_line = "Shot launched".to_string();
    }

    pub(crate) fn reset(&mut self) {
        self.phase = GamePhase::Aiming;
        self.shot = None;
        self.trail.clear();
        self.paused = false;
        self.status_line = "Reset".to_string();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceDragMode {
    Corner(usize),
    Surface,
    Rotate,
}

pub(crate) struct SurfaceEditor {
    pub(crate) drag_mode: Option<SurfaceDragMode>,
    pub(crate) hovered_corner: Option<usize>,
    pub(crate) hovered_surface: bool,
    pub(crate) hovered_rotate: bool,
    pub(crate) drag_start_mouse_world: Vec2,
    pub(crate) drag_start_corners: [Vec2; 4],
    pub(crate) rotate_center_world: Vec2,
    pub(crate) rotate_start_angle_rad: f32,
}

impl SurfaceEditor {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn is_dragging(&self) -> bool {
        self.drag_mode.is_some()
    }

    pub(crate) fn active_corner(&self) -> Option<usize> {
        match self.drag_mode {
            Some(SurfaceDragMode::Corner(idx)) => Some(idx),
            _ => None,
        }
    }

    pub(crate) fn active_rotate(&self) -> bool {
        matches!(self.drag_mode, Some(SurfaceDragMode::Rotate))
    }
}

pub(crate) struct LaunchEditor {
    pub(crate) active: bool,
    pub(crate) hovered: bool,
    pub(crate) ghost_screen: Vec2,
}

impl LaunchEditor {
    pub(crate) fn new() -> Self {
        Self {
            active: false,
            hovered: false,
            ghost_screen: Vec2::ZERO,
        }
    }
}

pub(crate) struct Prediction {
    pub(crate) points: Vec<Vec2>,
    pub(crate) range_m: f32,
    pub(crate) flight_time_s: f32,
    pub(crate) bounces: u32,
    pub(crate) outcome: StepOutcome,
}
