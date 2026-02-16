# parabolicRust
Parabolic projectile calculator and plot generator built in Rust.

## Overview
Given launch angle, velocity, and starting height, the program:
- calculates time of flight
- calculates horizontal distance (range)
- generates a PNG trajectory plot with labeled axes
- annotates landing with range (2 decimal places) and flight time (0.1s precision)

Generated files are named like:
```text
A75_V150_H600_trajectory_2-16-26.png
```

## Version History
- `v1`: basic CLI calculator (interactive input, time + distance output)
- `v2`: added `plotters` PNG graph generation
- `v3`: standardized chart framing with fixed distance:height window ratio
- `v4`: anchored plot axes at origin, timestamped/parameterized filenames, landing range label
- `v5`: thicker arc line, ground locked to chart bottom, two-line landing annotation (range + flight time)

## Prerequisites (Windows)
Install Rust:

```powershell
winget install Rustlang.Rustup
```

If `cargo` and `rustc` are not recognized:

```powershell
$env:Path += ";$HOME\.cargo\bin"
```

Persist PATH for future terminals:

```powershell
[Environment]::SetEnvironmentVariable(
  "Path",
  $env:Path + ";$HOME\.cargo\bin",
  "User"
)
```

For MSVC builds, ensure Visual Studio C++ Build Tools are installed and terminal environment includes `link.exe`.

## Run
From repo root:

```powershell
cargo run
```

Or pass values directly:

```powershell
cargo run -- 45 30 1.5
```

Input format:
```text
cargo run -- <angle_deg> <velocity_mps> <height_m>
```

## Test
```powershell
cargo test
```

## Publish
```powershell
git add .
git commit -m "Release v5 plotting updates"
git push origin main
```
