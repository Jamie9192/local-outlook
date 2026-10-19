$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
function Install-PackageIfNeeded($Command, $Id) {
  if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
    winget install --exact --id $Id --accept-package-agreements --accept-source-agreements
    if ($LASTEXITCODE -notin @(0, 3010)) { throw "Installation failed: $Id" }
    $env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
  }
}
Install-PackageIfNeeded 'node' 'OpenJS.NodeJS.LTS'
Install-PackageIfNeeded 'rustup' 'Rustlang.Rustup'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$cpp = if (Test-Path $vswhere) { & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath }
if (-not $cpp) {
  winget install --exact --id Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override '--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
  if ($LASTEXITCODE -notin @(0,3010)) { throw 'C++ Build Tools installation failed.' }
}
winget install --exact --id Microsoft.EdgeWebView2Runtime --accept-package-agreements --accept-source-agreements
# winget may report an already installed runtime; Tauri also bundles its bootstrapper.
rustup default stable-msvc
if ($LASTEXITCODE -ne 0) { throw 'Rust MSVC setup failed.' }
rustup component add rustfmt clippy
npm ci
if ($LASTEXITCODE -ne 0) { throw 'Node dependency installation failed.' }
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
if ($LASTEXITCODE -ne 0) { throw 'Rust dependency installation failed.' }
Write-Host 'Ready. Run npm run desktop -- dev to open Local Outlook.'
