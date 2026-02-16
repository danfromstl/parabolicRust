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
- `v6`: added an interactive real-time visualizer using `macroquad`
- `v7`: added web build pipeline (WASM + HTML), and upgraded interactive mode into a Moon level game with target + bounce

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

## Interactive Visualizer (macroquad)
Run the interactive app:

```powershell
cargo run --bin interactive_macroquad
```

Current mode:
- starts in `Moon Level 2` (`Bounce Into Target`)
- moon gravity, no drag, no wind
- requires at least one bounce on the fixed surface before a target hit counts

Controls:
- use sliders in the control panel for `Angle`, `Velocity`, and `Height`
- `Launch (Space)`: launch shot (or pause/resume while flying)
- `Reset (R)`: reset shot
- `Toggle Preview`: show/hide predicted path

## Web UI (v7)
Build the interactive app for browser:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_web.ps1
```

Then serve the `web/` folder:

```powershell
cd web
python -m http.server 8080
```

Open in Edge/Chrome:

```text
http://localhost:8080
```

Notes:
- Keep using a local server; opening `index.html` directly may fail due browser security rules for WASM/assets.
- `scripts/build_web.ps1` refreshes:
  - `web/interactive_macroquad.wasm`
  - `web/mq_js_bundle.js`
  - `web/assets/*`

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
