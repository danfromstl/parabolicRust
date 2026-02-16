$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

$cargoBin = Join-Path $HOME ".cargo\bin"
if ((Test-Path $cargoBin) -and -not (($env:Path -split ";") -contains $cargoBin)) {
    $env:Path = "$cargoBin;$env:Path"
}

Write-Host "[1/4] Ensuring wasm target is installed..."
rustup target add wasm32-unknown-unknown | Out-Host

Write-Host "[2/4] Building interactive_macroquad for web..."
cargo build --release --target wasm32-unknown-unknown --bin interactive_macroquad | Out-Host

$webDir = Join-Path $repoRoot "web"
New-Item -ItemType Directory -Path $webDir -Force | Out-Null

$wasmSource = Join-Path $repoRoot "target\wasm32-unknown-unknown\release\interactive_macroquad.wasm"
$wasmTarget = Join-Path $webDir "interactive_macroquad.wasm"
if (-not (Test-Path $wasmSource)) {
    throw "Build completed but wasm file was not found: $wasmSource"
}
Copy-Item $wasmSource $wasmTarget -Force

Write-Host "[3/4] Copying macroquad web loader..."
$mqJs = Get-ChildItem (Join-Path $HOME ".cargo\registry\src") -Recurse -Filter "mq_js_bundle.js" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if (-not $mqJs) {
    throw "Could not find mq_js_bundle.js in Cargo registry. Try rebuilding once with 'cargo build'."
}
Copy-Item $mqJs.FullName (Join-Path $webDir "mq_js_bundle.js") -Force

Write-Host "[4/4] Copying assets for web runtime..."
$assetsSource = Join-Path $repoRoot "assets"
$assetsTarget = Join-Path $webDir "assets"
if (Test-Path $assetsTarget) {
    Remove-Item $assetsTarget -Recurse -Force
}
if (Test-Path $assetsSource) {
    Copy-Item $assetsSource $assetsTarget -Recurse -Force
}

Write-Host ""
Write-Host "Web build complete."
Write-Host "Output:"
Write-Host "  $wasmTarget"
Write-Host "  $(Join-Path $webDir 'mq_js_bundle.js')"
Write-Host ""
Write-Host "Next:"
Write-Host "  cd web"
Write-Host "  python -m http.server 8080"
Write-Host "  open http://localhost:8080"
