# RustPlayer one-shot Windows build: downloads libmpv + font, builds the exe,
# creates dist\RustPlayer-portable.zip and (if Inno Setup is installed) dist\RustPlayer-Setup-x.y.z.exe
param([switch]$NoInstaller)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$root = $PSScriptRoot
Set-Location $root
function Step($m) { Write-Host "`n==> $m" -ForegroundColor Yellow }

$deps = Join-Path $root 'deps'
$mpvDir = Join-Path $deps 'mpv'
New-Item -ItemType Directory -Force $deps, $mpvDir | Out-Null

# 1. Toolchain checks
Step "Checking tools"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Rust is not installed. Install it from https://rustup.rs and run this script again."
}
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere)) {
    throw "Visual Studio Build Tools not found. Install them with the 'Desktop development with C++' workload."
}
$libExe  = & $vswhere -latest -products * -find 'VC\Tools\MSVC\**\bin\Hostx64\x64\lib.exe' | Select-Object -First 1
$dumpbin = & $vswhere -latest -products * -find 'VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe' | Select-Object -First 1
if (-not $libExe) { throw "lib.exe not found. Add the 'Desktop development with C++' workload in the VS installer." }

# 2. libmpv (shinchiro builds)
$dll = Get-ChildItem $mpvDir -Filter 'libmpv*.dll' -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $dll) {
    Step "Downloading libmpv"
    $rel = Invoke-RestMethod 'https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest' -Headers @{ 'User-Agent' = 'rustplayer-build' }
    $asset = $rel.assets | Where-Object { $_.name -match '^mpv-dev-x86_64-\d.*\.7z$' } | Select-Object -First 1
    if (-not $asset) { throw "No mpv-dev-x86_64 package found in the latest shinchiro release." }
    $archive = Join-Path $deps $asset.name
    Invoke-WebRequest $asset.browser_download_url -OutFile $archive
    $7z = (Get-Command 7z -ErrorAction SilentlyContinue).Source
    if (-not $7z -and (Test-Path "$env:ProgramFiles\7-Zip\7z.exe")) { $7z = "$env:ProgramFiles\7-Zip\7z.exe" }
    if (-not $7z) {
        $7z = Join-Path $deps '7zr.exe'
        Invoke-WebRequest 'https://www.7-zip.org/a/7zr.exe' -OutFile $7z
    }
    & $7z x $archive "-o$mpvDir" -y | Out-Null
    $dll = Get-ChildItem $mpvDir -Filter 'libmpv*.dll' | Select-Object -First 1
    if (-not $dll) { throw "libmpv dll not found after extracting $($asset.name)" }
}

# 3. Import library for MSVC
if (-not (Test-Path "$mpvDir\mpv.lib")) {
    Step "Creating mpv.lib"
    $def = Join-Path $mpvDir 'mpv.def'
    if (-not (Test-Path $def)) {
        if (-not $dumpbin) { throw "dumpbin.exe not found; cannot generate mpv.def" }
        $exports = & $dumpbin /exports $dll.FullName | ForEach-Object {
            if ($_ -match '^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]{8}\s+(\S+)') { $Matches[1] }
        }
        @('EXPORTS') + $exports | Set-Content $def -Encoding ascii
    }
    & $libExe /nologo "/def:$def" "/name:$($dll.Name)" /machine:x64 "/out:$mpvDir\mpv.lib" | Out-Null
    if (-not (Test-Path "$mpvDir\mpv.lib")) { throw "lib.exe failed to create mpv.lib" }
}

# 4. Persian font
$fontDir = Join-Path $root 'assets\fonts'
New-Item -ItemType Directory -Force $fontDir | Out-Null
$font = Join-Path $fontDir 'Vazirmatn-Regular.ttf'
if (-not (Test-Path $font)) {
    Step "Downloading Vazirmatn font"
    try {
        Invoke-WebRequest 'https://github.com/rastikerdar/vazirmatn/raw/master/fonts/ttf/Vazirmatn-Regular.ttf' -OutFile $font
    } catch {
        Write-Warning "Font download failed. The app will fall back to Tahoma."
    }
}

# 5. Build
Step "Building RustPlayer (release)"
$env:MPV_DIR = $mpvDir
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed (see errors above)" }

# 6. Stage + portable zip
Step "Packaging"
$dist = Join-Path $root 'dist'
$app = Join-Path $dist 'app'
if (Test-Path $app) { Remove-Item -Recurse -Force $app }
New-Item -ItemType Directory -Force "$app\fonts" | Out-Null
Copy-Item 'target\release\rustplayer.exe' $app
Copy-Item $dll.FullName $app
if (Test-Path $font) { Copy-Item $font "$app\fonts" }
$ver = (Select-String -Path 'Cargo.toml' -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
foreach ($f in 'README.md', 'LICENSE', 'THIRD-PARTY-NOTICES.md') { if (Test-Path $f) { Copy-Item $f $app } }
$zip = Join-Path $dist "RustPlayer-$ver-portable-win64.zip"
Get-ChildItem $dist -Filter 'RustPlayer-*portable*.zip' -ErrorAction SilentlyContinue | Remove-Item
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path "$app\*" -DestinationPath $zip

# 7. Installer
if (-not $NoInstaller) {
    $iscc = (Get-Command iscc -ErrorAction SilentlyContinue).Source
    foreach ($p in "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") {
        if (-not $iscc -and (Test-Path $p)) { $iscc = $p }
    }
    if (-not $iscc) {
        Write-Warning "Inno Setup not found, so only the portable zip was made. Install it with: winget install JRSoftware.InnoSetup"
    } else {
        Step "Building installer"
        & $iscc /Q 'installer\rustplayer.iss'
        if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed" }
    }
}
Step "Done. Look in the dist folder."
Get-ChildItem $dist -File | ForEach-Object { Write-Host "   $($_.Name)" -ForegroundColor Green }
