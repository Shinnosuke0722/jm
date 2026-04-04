# jm installer for Windows — https://github.com/jm-sh/jm
#
# Usage:
#   irm https://raw.githubusercontent.com/jm-sh/jm/main/install.ps1 | iex
#   # Or with options:
#   & ([scriptblock]::Create((irm https://raw.githubusercontent.com/jm-sh/jm/main/install.ps1))) -Version v0.2.0

param(
    [string]$Version = "",
    [string]$InstallDir = ""
)

$ErrorActionPreference = "Stop"

# ─── Configuration ──────────────────────────────────────────────────────────

$Repo = "jm-sh/jm"
$BinaryName = "jm.exe"
$DefaultInstallDir = "$env:USERPROFILE\.jm\bin"

if (-not $InstallDir) {
    $InstallDir = $DefaultInstallDir
}

# ─── Helpers ────────────────────────────────────────────────────────────────

function Write-Info($msg)    { Write-Host "  > " -ForegroundColor Blue -NoNewline; Write-Host $msg }
function Write-Success($msg) { Write-Host "  > " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Err($msg)     { Write-Host "  error: " -ForegroundColor Red -NoNewline; Write-Host $msg; exit 1 }

# ─── Platform detection ────────────────────────────────────────────────────

function Get-Arch {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        "X64"   { return "x86_64" }
        "Arm64" { return "aarch64" }
        default { Write-Err "Unsupported architecture: $arch" }
    }
}

# ─── Version resolution ────────────────────────────────────────────────────

function Get-LatestVersion {
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "jm-installer" }
        return $release.tag_name
    }
    catch {
        Write-Err "Failed to fetch latest release. Check your network or use -Version to specify."
    }
}

# ─── Main ───────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "  jm installer" -ForegroundColor White
Write-Host ""

$Arch = Get-Arch
Write-Info "Detected platform: windows $Arch"

# Resolve version
if (-not $Version) {
    Write-Info "Fetching latest version..."
    $Version = Get-LatestVersion
}
Write-Info "Installing version: $Version"

# Determine download URL
$Artifact = "jm-windows-${Arch}"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$Artifact.zip"
Write-Info "Downloading from: $DownloadUrl"

# Download to temp
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "jm-install-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

try {
    $ZipPath = Join-Path $TmpDir "$Artifact.zip"
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
    }
    catch {
        Write-Err "Download failed. Check that version $Version exists and has release assets."
    }

    # Verify checksum
    $ChecksumsUrl = "https://github.com/$Repo/releases/download/$Version/sha256sums.txt"
    try {
        $Checksums = Invoke-RestMethod -Uri $ChecksumsUrl -Headers @{ "User-Agent" = "jm-installer" }
        $ExpectedLine = ($Checksums -split "`n") | Where-Object { $_ -match "$Artifact.zip" } | Select-Object -First 1
        if ($ExpectedLine) {
            $Expected = ($ExpectedLine -split "\s+")[0]
            $Actual = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
            if ($Expected -ne $Actual) {
                Write-Err "Checksum mismatch!`n  Expected: $Expected`n  Actual:   $Actual"
            }
            Write-Success "Checksum verified"
        }
    }
    catch {
        Write-Info "Skipping checksum verification (could not fetch checksums)"
    }

    # Extract
    Write-Info "Extracting..."
    Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

    # Install
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    Move-Item -Path (Join-Path $TmpDir $BinaryName) -Destination (Join-Path $InstallDir $BinaryName) -Force
    Write-Success "Installed jm to $InstallDir\$BinaryName"

    # Add to PATH via User environment variable
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
        Write-Success "Added $InstallDir to your PATH (takes effect in new terminals)"
        Write-Info "Run this to use jm in the current terminal:"
        Write-Host ""
        Write-Host "    `$env:Path = `"$InstallDir;`$env:Path`""
    }

    # Shell integration hint
    Write-Host ""
    Write-Info "Set up shell integration by adding to your PowerShell profile (`$PROFILE):"
    Write-Host ""
    Write-Host "    jm shell init powershell | Invoke-Expression"
    Write-Host ""
    Write-Success "Done! Run 'jm --help' to get started."
    Write-Host ""
}
finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
