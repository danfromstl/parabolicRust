use chrono::{Datelike, Local};
use plotters::prelude::*;
use std::env;
use std::io::{self, Write};

const G: f64 = 9.8; // m/s^2
const PLOT_WIDTH: u32 = 1500;
const PLOT_HEIGHT: u32 = 780;
const CHART_MARGIN: u32 = 20;
const X_LABEL_AREA: u32 = 60;
const Y_LABEL_AREA: u32 = 100;
const TRAJECTORY_SAMPLES: usize = 500;
const DISTANCE_TO_HEIGHT_RATIO: f64 = 2.0; // 1:2 height:distance window
const TRAJECTORY_LINE_WIDTH: u32 = 3;

#[derive(Clone, Copy, Debug)]
struct Inputs {
    angle_deg: f64,
    speed_mps: f64,
    height_m: f64,
}

fn parse_f64(value: &str, label: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|_| format!("Invalid {label}: '{value}'. Expected a number."))
}

fn read_f64(prompt: &str) -> Result<f64, String> {
    loop {
        print!("{prompt}");
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {e}"))?;

        let mut line = String::new();
        let bytes = io::stdin()
            .read_line(&mut line)
            .map_err(|e| format!("Could not read input: {e}"))?;

        if bytes == 0 {
            return Err("Input ended unexpectedly (EOF).".to_string());
        }

        match line.trim().parse::<f64>() {
            Ok(v) => return Ok(v),
            Err(_) => eprintln!("Please enter a valid number (e.g., 45 or 12.5)."),
        }
    }
}

fn get_inputs_from_user() -> Result<Inputs, String> {
    Ok(Inputs {
        angle_deg: read_f64("Angle (degrees): ")?,
        speed_mps: read_f64("Velocity (m/s): ")?,
        height_m: read_f64("Height (m): ")?,
    })
}

fn get_inputs_from_args(args: &[String]) -> Result<Inputs, String> {
    if args.len() != 4 {
        return Err(
            "Expected exactly 3 arguments: <angle_deg> <velocity_mps> <height_m>.".to_string(),
        );
    }

    Ok(Inputs {
        angle_deg: parse_f64(&args[1], "angle")?,
        speed_mps: parse_f64(&args[2], "velocity")?,
        height_m: parse_f64(&args[3], "height")?,
    })
}

fn velocity_components(inputs: Inputs) -> (f64, f64) {
    let theta = inputs.angle_deg.to_radians();
    let vx = inputs.speed_mps * theta.cos();
    let vy = inputs.speed_mps * theta.sin();
    (vx, vy)
}

fn trajectory_at_time(inputs: Inputs, time_s: f64) -> (f64, f64) {
    let (vx, vy) = velocity_components(inputs);
    let x = vx * time_s;
    let y = inputs.height_m + (vy * time_s) - (0.5 * G * time_s * time_s);
    (x, y)
}

fn flight_time_and_range(inputs: Inputs) -> Result<(f64, f64), String> {
    if !inputs.angle_deg.is_finite()
        || !inputs.speed_mps.is_finite()
        || !inputs.height_m.is_finite()
    {
        return Err("Inputs must be finite numbers.".to_string());
    }
    if inputs.speed_mps < 0.0 {
        return Err("Velocity cannot be negative.".to_string());
    }

    let (_, vy) = velocity_components(inputs);

    let disc = vy * vy + 2.0 * G * inputs.height_m;
    if disc < 0.0 {
        return Err(format!(
            "No real landing time: vy^2 + 2*g*h is negative ({disc})."
        ));
    }

    let t_land = (vy + disc.sqrt()) / G;
    if t_land < 0.0 {
        return Err(format!(
            "Landing time computed as negative ({t_land}). Check your inputs."
        ));
    }

    let (range, _) = trajectory_at_time(inputs, t_land);
    Ok((t_land, range))
}

fn sample_trajectory(inputs: Inputs, time_of_flight_s: f64, samples: usize) -> Vec<(f64, f64)> {
    let sample_count = samples.max(2);
    (0..=sample_count)
        .map(|i| {
            let t = (i as f64 * time_of_flight_s) / sample_count as f64;
            trajectory_at_time(inputs, t)
        })
        .collect()
}

fn axis_bounds(points: &[(f64, f64)]) -> ((f64, f64), (f64, f64)) {
    let max_x = points
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    let max_y = points
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);

    let raw_x_span = max_x.max(1.0);
    let raw_y_span = max_y.max(1.0);
    let x_pad = raw_x_span * 0.06;
    let y_pad = raw_y_span * 0.10;

    let mut x_span = (max_x + x_pad).max(1.0);
    let mut y_span = (max_y + y_pad).max(1.0);

    // Expand the smaller span so every image keeps the same data-window ratio.
    if x_span / y_span < DISTANCE_TO_HEIGHT_RATIO {
        x_span = y_span * DISTANCE_TO_HEIGHT_RATIO;
    } else {
        y_span = x_span / DISTANCE_TO_HEIGHT_RATIO;
    }

    let x_min = 0.0;
    let x_max = x_span;
    let y_min = 0.0;
    let y_max = y_span;

    ((x_min, x_max), (y_min, y_max))
}

fn format_value_for_filename(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let mut s = if rounded.fract().abs() < 1e-9 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };
    if let Some(stripped) = s.strip_prefix('-') {
        s = format!("neg{stripped}");
    }
    s.replace('.', "p")
}

fn build_output_image_name(inputs: Inputs) -> String {
    let now = Local::now();
    let yy = ((now.year() % 100) + 100) % 100;
    format!(
        "A{}_V{}_H{}_trajectory_{}-{}-{:02}.png",
        format_value_for_filename(inputs.angle_deg),
        format_value_for_filename(inputs.speed_mps),
        format_value_for_filename(inputs.height_m),
        now.month(),
        now.day(),
        yy
    )
}

fn save_trajectory_plot(
    inputs: Inputs,
    time_of_flight_s: f64,
    horizontal_distance_m: f64,
    output_path: &str,
) -> Result<(), String> {
    let points = sample_trajectory(inputs, time_of_flight_s, TRAJECTORY_SAMPLES);
    let ((x_min, x_max), (y_min, y_max)) = axis_bounds(&points);

    let launch = points.first().copied().unwrap_or((0.0, inputs.height_m));
    let landing = points.last().copied().unwrap_or((0.0, 0.0));

    let root = BitMapBackend::new(output_path, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| format!("Failed to clear plotting area: {e:?}"))?;

    let mut chart = ChartBuilder::on(&root)
        .margin(CHART_MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| format!("Failed to build plot axes: {e:?}"))?;

    chart
        .configure_mesh()
        .x_desc("Horizontal Distance (m)")
        .y_desc("Height (m)")
        .x_labels(12)
        .y_labels(12)
        .axis_desc_style(("Segoe UI", 20).into_font())
        .label_style(("Segoe UI", 14).into_font())
        .light_line_style(RGBColor(220, 220, 220))
        .bold_line_style(RGBColor(190, 190, 190))
        .draw()
        .map_err(|e| format!("Failed to draw grid/axes: {e:?}"))?;

    chart
        .draw_series(LineSeries::new(
            points.iter().copied(),
            ShapeStyle::from(&BLUE.mix(0.9)).stroke_width(TRAJECTORY_LINE_WIDTH),
        ))
        .map_err(|e| format!("Failed to draw trajectory line: {e:?}"))?;

    chart
        .draw_series(LineSeries::new(
            [(x_min, 0.0), (x_max, 0.0)],
            &BLACK.mix(0.4),
        ))
        .map_err(|e| format!("Failed to draw ground reference line: {e:?}"))?;

    chart
        .draw_series(std::iter::once(Circle::new(launch, 5, RED.filled())))
        .map_err(|e| format!("Failed to draw launch point: {e:?}"))?;

    chart
        .draw_series(std::iter::once(Circle::new(landing, 5, GREEN.filled())))
        .map_err(|e| format!("Failed to draw landing point: {e:?}"))?;

    let x_span = x_max - x_min;
    let y_span = y_max - y_min;
    let distance_label = format!("Range: {:.2} m", horizontal_distance_m.abs());
    let time_label = format!("Flight time: {:.1} s", time_of_flight_s);
    let label_x = (landing.0 + (0.02 * x_span)).min(x_max - (0.01 * x_span));
    let label_y = landing.1 + (0.04 * y_span);

    chart
        .draw_series(std::iter::once(Text::new(
            distance_label,
            (label_x, label_y + (0.035 * y_span)),
            ("Segoe UI", 16).into_font(),
        )))
        .map_err(|e| format!("Failed to draw landing label: {e:?}"))?;

    chart
        .draw_series(std::iter::once(Text::new(
            time_label,
            (label_x, label_y),
            ("Segoe UI", 16).into_font(),
        )))
        .map_err(|e| format!("Failed to draw flight-time label: {e:?}"))?;

    root.present()
        .map_err(|e| format!("Failed to write image file: {e:?}"))?;

    Ok(())
}

fn print_usage(program: &str) {
    println!("Usage:");
    println!("  {program}");
    println!("  {program} <angle_deg> <velocity_mps> <height_m>");
    println!();
    println!("Examples:");
    println!("  {program}");
    println!("  {program} 45 30 1.5");
    println!();
    println!("The program saves a PNG plot named like:");
    println!("  A75_V150_H600_trajectory_2-16-26.png");
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage(&args[0]);
        return Ok(());
    }

    let inputs = if args.len() == 1 {
        get_inputs_from_user()?
    } else {
        get_inputs_from_args(&args)?
    };

    let (time, distance) = flight_time_and_range(inputs)?;
    let output_image = build_output_image_name(inputs);

    println!("\nTime of flight: {:.4} s", time);
    println!("Horizontal distance: {:.4} m", distance);

    save_trajectory_plot(inputs, time, distance, &output_image)?;
    println!("Saved plot: {output_image}");

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        print_usage("cargo run --");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Inputs, axis_bounds, flight_time_and_range, sample_trajectory};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "actual={actual}, expected={expected}, tolerance={tolerance}"
        );
    }

    #[test]
    fn computes_known_range_for_flat_ground() {
        let (time, distance) = flight_time_and_range(Inputs {
            angle_deg: 45.0,
            speed_mps: 10.0,
            height_m: 0.0,
        })
        .expect("calculation should succeed");

        assert_close(time, 1.4431, 0.001);
        assert_close(distance, 10.2041, 0.001);
    }

    #[test]
    fn zero_velocity_has_zero_range() {
        let (time, distance) = flight_time_and_range(Inputs {
            angle_deg: 10.0,
            speed_mps: 0.0,
            height_m: 2.0,
        })
        .expect("calculation should succeed");

        assert_close(time, 0.6389, 0.001);
        assert_close(distance, 0.0, 0.0001);
    }

    #[test]
    fn rejects_impossible_landing_time() {
        let err = flight_time_and_range(Inputs {
            angle_deg: 0.0,
            speed_mps: 1.0,
            height_m: -10.0,
        })
        .expect_err("calculation should fail");

        assert!(err.contains("No real landing time"));
    }

    #[test]
    fn sampled_trajectory_starts_and_ends_at_expected_points() {
        let inputs = Inputs {
            angle_deg: 45.0,
            speed_mps: 30.0,
            height_m: 1.5,
        };
        let (time, _) = flight_time_and_range(inputs).expect("calculation should succeed");
        let points = sample_trajectory(inputs, time, 100);

        let first = points.first().copied().expect("has first point");
        let last = points.last().copied().expect("has last point");
        assert_close(first.0, 0.0, 1e-9);
        assert_close(first.1, 1.5, 1e-9);
        assert_close(last.1, 0.0, 0.01);
    }

    #[test]
    fn plot_window_keeps_x_origin_on_left() {
        let inputs = Inputs {
            angle_deg: 60.0,
            speed_mps: 25.0,
            height_m: 10.0,
        };
        let (time, _) = flight_time_and_range(inputs).expect("calculation should succeed");
        let points = sample_trajectory(inputs, time, 100);
        let ((x_min, x_max), _) = axis_bounds(&points);

        assert_close(x_min, 0.0, 1e-9);
        assert!(x_max > 0.0);
    }

    #[test]
    fn plot_window_keeps_ground_on_bottom() {
        let inputs = Inputs {
            angle_deg: 45.0,
            speed_mps: 20.0,
            height_m: 3.0,
        };
        let (time, _) = flight_time_and_range(inputs).expect("calculation should succeed");
        let points = sample_trajectory(inputs, time, 100);
        let (_, (y_min, y_max)) = axis_bounds(&points);

        assert_close(y_min, 0.0, 1e-9);
        assert!(y_max > 0.0);
    }
}
