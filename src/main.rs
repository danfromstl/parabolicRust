use plotters::prelude::*;
use std::env;
use std::io::{self, Write};

const G: f64 = 9.8; // m/s^2
const OUTPUT_IMAGE: &str = "trajectory.png";
const PLOT_WIDTH: u32 = 1280;
const PLOT_HEIGHT: u32 = 720;
const TRAJECTORY_SAMPLES: usize = 500;

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
    let min_y = points
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min)
        .min(0.0);
    let max_y = points
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);

    let x_min = 0.0;
    let x_max = if max_x > 0.0 { max_x * 1.08 } else { 1.0 };

    let y_span = (max_y - min_y).abs();
    let y_pad = if y_span > 0.0 { y_span * 0.12 } else { 1.0 };
    let y_min = min_y - (0.25 * y_pad);
    let y_max = max_y + y_pad;

    ((x_min, x_max), (y_min, y_max))
}

fn save_trajectory_plot(
    inputs: Inputs,
    time_of_flight_s: f64,
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
        .caption("Parabolic Trajectory", ("Segoe UI", 34).into_font())
        .margin(20)
        .x_label_area_size(55)
        .y_label_area_size(70)
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
        .draw_series(LineSeries::new(points.iter().copied(), &BLUE.mix(0.9)))
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
    println!("The program saves a PNG plot to {OUTPUT_IMAGE}.");
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

    println!("\nTime of flight: {:.4} s", time);
    println!("Horizontal distance: {:.4} m", distance);

    save_trajectory_plot(inputs, time, OUTPUT_IMAGE)?;
    println!("Saved plot: {OUTPUT_IMAGE}");

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
    use super::{Inputs, flight_time_and_range, sample_trajectory};

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
}
