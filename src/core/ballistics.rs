pub const EARTH_GRAVITY_MPS2: f64 = 9.8;

#[derive(Clone, Copy, Debug)]
pub struct LaunchInputs {
    pub angle_deg: f64,
    pub speed_mps: f64,
    pub height_m: f64,
}

pub fn velocity_components(inputs: LaunchInputs) -> (f64, f64) {
    let theta = inputs.angle_deg.to_radians();
    let vx = inputs.speed_mps * theta.cos();
    let vy = inputs.speed_mps * theta.sin();
    (vx, vy)
}

pub fn trajectory_at_time(inputs: LaunchInputs, time_s: f64) -> (f64, f64) {
    let (vx, vy) = velocity_components(inputs);
    let x = vx * time_s;
    let y = inputs.height_m + (vy * time_s) - (0.5 * EARTH_GRAVITY_MPS2 * time_s * time_s);
    (x, y)
}

pub fn flight_time_and_range(inputs: LaunchInputs) -> Result<(f64, f64), String> {
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
    let disc = vy * vy + 2.0 * EARTH_GRAVITY_MPS2 * inputs.height_m;
    if disc < 0.0 {
        return Err(format!(
            "No real landing time: vy^2 + 2*g*h is negative ({disc})."
        ));
    }

    let t_land = (vy + disc.sqrt()) / EARTH_GRAVITY_MPS2;
    if t_land < 0.0 {
        return Err(format!(
            "Landing time computed as negative ({t_land}). Check your inputs."
        ));
    }

    let (range, _) = trajectory_at_time(inputs, t_land);
    Ok((t_land, range))
}

pub fn sample_trajectory(
    inputs: LaunchInputs,
    time_of_flight_s: f64,
    samples: usize,
) -> Vec<(f64, f64)> {
    let sample_count = samples.max(2);
    (0..=sample_count)
        .map(|i| {
            let t = (i as f64 * time_of_flight_s) / sample_count as f64;
            trajectory_at_time(inputs, t)
        })
        .collect()
}
