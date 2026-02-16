# parabolicRust
Parabolic arc distance calculator written in Rust.

## What It Does
Given launch angle, velocity, and starting height, it calculates:
- time of flight
- horizontal distance
- a trajectory plot image with names like `A75_V150_H600_trajectory_2-16-26.png`
- a consistent chart window ratio (1:2 height:distance) for easier visual comparison

## Prerequisites (Windows)
Install Rust with `rustup` if needed:

```powershell
winget install Rustlang.Rustup
```

If `cargo`/`rustc` are not recognized, add Cargo to PATH:

```powershell
$env:Path += ";$HOME\.cargo\bin"
```

To persist PATH for future terminals:

```powershell
[Environment]::SetEnvironmentVariable(
  "Path",
  $env:Path + ";$HOME\.cargo\bin",
  "User"
)
```

Close and reopen your terminal after setting persistent PATH.

## Run
From repo root:

```powershell
cargo run
```

Or pass values directly:

```powershell
cargo run -- 45 30 1.5
```

Format is:

```text
cargo run -- <angle_deg> <velocity_mps> <height_m>
```

After each run, the program writes a PNG in the repo root, for example:

```text
A75_V150_H600_trajectory_2-16-26.png
```

## Test
```powershell
cargo test
```

## Publish To GitHub
```powershell
git add .
git commit -m "Create working parabolic arc Rust CLI"
git push origin main
```
