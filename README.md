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
- `v8 - Phase 1`: began structural refactor planning baseline (behavior-preserving organization pass before optimization/feature changes)
- `v8 - Phase 2`: split the interactive macroquad binary into focused modules (`app`, `input`, `physics`, `render`, `model`, `constants`) with no intended behavior changes
- `v8 - Phase 3`: extracted runtime state and gameplay/control flows into dedicated modules (`state`, `gameplay`, `controls`) so `app` is now mostly orchestration
- `v8 - Phase 4`: extracted HUD/status/range-label composition into a dedicated `hud` module so `app` only coordinates update + render flow
- `v9`: added an Earth campaign (4 levels) before Moon with Earth gravity + drag + random mild horizontal wind, and added a top-right success CTA button (`Next Level`)

## v8 - Phase 1 Structure
This phase is intentionally behavior-preserving. No gameplay/plot logic was redesigned; shared core logic was moved into reusable library modules so future refactors can happen safely.

Current shared module layout:
```text
src/
  lib.rs
  core/
    mod.rs
    ballistics.rs   # shared launch/trajectory/flight calculations (f64)
    window.rs       # shared fixed-ratio axis window helpers (f64 + f32)
```

Notes:
- `src/main.rs` now uses `core::ballistics` and `core::window` instead of local duplicate math.
- `src/bin/interactive_macroquad/main.rs` (through module wiring) now uses `core::window` for the same axis scaling rule source.

v8 Phase 2 interactive layout:
```text
src/bin/interactive_macroquad/
  main.rs
  app.rs
  constants.rs
  controls.rs
  gameplay.rs
  hud.rs
  input.rs
  model.rs
  physics.rs
  render.rs
  state.rs
```

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
- starts in a 4-level Earth campaign, then continues into a 4-level Moon campaign
- Earth levels use:
  - gravity `9.8 m/s^2`
  - constant linear drag (`drag_linear = 0.015`)
  - mild random horizontal wind per level (forward or backward)
- Moon levels use low gravity with no wind/drag
- level progression unlocks as you clear each level

Controls:
- use sliders in the control panel for `Angle`, `Velocity`, and `Height`
- use `Simulation Speed` slider (`0.5x` to `5.0x`)
- drag the launch dot and pull a ghost handle left/up/down to set launch angle + velocity
- `W/S`: increase/decrease height (when mouse is not held)
- `A/D`: decrease/increase velocity (when mouse is not held)
- `Launch (Space)`: launch shot (or pause/resume while flying)
- `Reset (R)`: reset shot
- `Toggle Preview`: show/hide predicted path
- `Prev Level (P)` / `Next Level (N)`: navigate unlocked levels
- after a successful clear, a large top-right `Next Level` button appears below the level label
- Moon Levels 2 and 4: hover the bounce surface to reveal corner + rotation handles
- drag any corner to reshape, drag inside the surface to move it, or drag the rotation handle to rotate it

Startup:
- title screen appears first with a `Start Game` button (or press `Enter`/`Space`)

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
