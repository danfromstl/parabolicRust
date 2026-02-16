use std::io::{self, Write};

const G: f64 = 9.8; // m/s^2

fn read_f64(prompt: &str) -> f64 {
    loop {
        print!("{prompt}");
        // Ensure the prompt shows up before we read input
        io::stdout().flush().expect("Failed to flush stdout");

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            eprintln!("Could not read input. Try again.");
            continue;
        }

        match line.trim().parse::<f64>() {
            Ok(v) => return v,
            Err(_) => eprintln!("Please enter a valid number (e.g., 45 or 12.5)."),
        }
    }
}

fn main() {
    let angle_deg = read_f64("Angle (degrees): ");
    let speed = read_f64("Velocity (m/s): ");
    let height = read_f64("Height (m): ");

    let theta = angle_deg.to_radians();
    let vx = speed * theta.cos();
    let vy = speed * theta.sin();

    // Discriminant for 0.5*g*t^2 - vy*t - height = 0
    let disc = vy * vy + 2.0 * G * height;

    if disc < 0.0 {
        eprintln!(
            "No real landing time: vy^2 + 2*g*h is negative ({disc}). Check your inputs."
        );
        return;
    }

    let t_land = (vy + disc.sqrt()) / G;

    if t_land < 0.0 {
        eprintln!("Landing time computed as negative ({t_land}). Check your inputs.");
        return;
    }

    let range = vx * t_land;

    println!("\nTime of flight: {:.4} s", t_land);
    println!("Horizontal distance: {:.4} m", range);
}
