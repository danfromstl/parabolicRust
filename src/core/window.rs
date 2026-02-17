pub const DISTANCE_TO_HEIGHT_RATIO: f64 = 2.0; // x:y data window ratio

const X_PADDING_RATIO: f64 = 0.06;
const Y_PADDING_RATIO: f64 = 0.10;

pub fn fixed_ratio_axis_window_f64(raw_max_x: f64, raw_max_y: f64) -> (f64, f64) {
    let raw_x_span = raw_max_x.max(1.0);
    let raw_y_span = raw_max_y.max(1.0);
    let x_pad = raw_x_span * X_PADDING_RATIO;
    let y_pad = raw_y_span * Y_PADDING_RATIO;

    let mut x_span = (raw_max_x + x_pad).max(1.0);
    let mut y_span = (raw_max_y + y_pad).max(1.0);

    if x_span / y_span < DISTANCE_TO_HEIGHT_RATIO {
        x_span = y_span * DISTANCE_TO_HEIGHT_RATIO;
    } else {
        y_span = x_span / DISTANCE_TO_HEIGHT_RATIO;
    }

    (x_span, y_span)
}

pub fn fixed_ratio_axis_window_f32(raw_max_x: f32, raw_max_y: f32) -> (f32, f32) {
    let raw_x_span = raw_max_x.max(1.0);
    let raw_y_span = raw_max_y.max(1.0);
    let x_pad = raw_x_span * X_PADDING_RATIO as f32;
    let y_pad = raw_y_span * Y_PADDING_RATIO as f32;

    let mut x_span = (raw_max_x + x_pad).max(1.0);
    let mut y_span = (raw_max_y + y_pad).max(1.0);

    if x_span / y_span < DISTANCE_TO_HEIGHT_RATIO as f32 {
        x_span = y_span * DISTANCE_TO_HEIGHT_RATIO as f32;
    } else {
        y_span = x_span / DISTANCE_TO_HEIGHT_RATIO as f32;
    }

    (x_span, y_span)
}
