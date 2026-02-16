use std::env;
use std::io::{self, Write};

const G: f64 = 9.8; // m/s^2

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

    let theta = inputs.angle_deg.to_radians();
    let vx = inputs.speed_mps * theta.cos();
    let vy = inputs.speed_mps * theta.sin();

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

    Ok((t_land, vx * t_land))
}

fn print_usage(program: &str) {
    println!("Usage:");
    println!("  {program}");
    println!("  {program} <angle_deg> <velocity_mps> <height_m>");
    println!();
    println!("Examples:");
    println!("  {program}");
    println!("  {program} 45 30 1.5");
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
    use super::{Inputs, flight_time_and_range};

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
}
