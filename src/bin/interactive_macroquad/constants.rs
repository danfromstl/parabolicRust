use macroquad::prelude::Color;

pub const INITIAL_WINDOW_WIDTH: i32 = 1920;
pub const INITIAL_WINDOW_HEIGHT: i32 = 1080;
pub const MSAA_SAMPLES: i32 = 4;
pub const UI_FONT_PATH: &str = "assets/fonts/Lato-Regular.ttf";

pub const LEFT_MARGIN: f32 = 120.0;
pub const RIGHT_MARGIN: f32 = 30.0;
pub const TOP_MARGIN: f32 = 140.0;
pub const BOTTOM_MARGIN: f32 = 130.0;

pub const TITLE_Y: f32 = 46.0;
pub const CONTROLS_Y: f32 = 92.0;
pub const TRAJECTORY_SAMPLES: usize = 320;
pub const FIXED_STEP_S: f32 = 1.0 / 240.0;
pub const MAX_SIM_TIME_S: f32 = 60.0;
pub const X_GRID_LINES: usize = 10;
pub const Y_GRID_LINES: usize = 8;
pub const TITLE_SCREEN_BG: Color = Color::new(0.92, 0.93, 0.95, 1.0);
pub const START_BUTTON_COLOR: Color = Color::new(0.14, 0.45, 0.95, 1.0);
pub const START_BUTTON_TEXT: &str = "Start Game";
pub const SURFACE_HANDLE_RADIUS: f32 = 8.0;
pub const ROTATE_HANDLE_RADIUS: f32 = 9.0;
pub const ROTATE_HANDLE_STICK_PX: f32 = 34.0;
pub const LAUNCH_HANDLE_RADIUS: f32 = 8.0;
pub const LAUNCH_GHOST_RADIUS: f32 = 7.0;
pub const LAUNCH_DRAG_MIN_PX: f32 = 10.0;
pub const LAUNCH_GHOST_BELOW_AXIS_PX: f32 = 220.0;
pub const HEIGHT_KEY_RATE_MPS: f32 = 90.0;
pub const VELOCITY_KEY_RATE_MPS: f32 = 140.0;
pub const SLINGSHOT_VERTICAL_MIRROR: bool = true;
