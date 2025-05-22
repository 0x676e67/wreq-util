param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("windows", "linux")]
    [string]$Target = "windows"
)

$ErrorActionPreference = "Stop"

# Ensure required features are enabled
$Features = "cli,emulation,gzip,brotli,deflate,zstd,rquest/full"

# Set target based on OS
$RustTarget = if ($Target -eq "windows") {
    "x86_64-pc-windows-msvc"
} else {
    "x86_64-unknown-linux-gnu"
}

# Install target if not already installed
Write-Host "Installing target $RustTarget..."
rustup target add $RustTarget

# Build the release binary
Write-Host "Building release binary for $RustTarget..."
cargo build --bin rquest_runner --release --target $RustTarget --features $Features

# Create dist directory if it doesn't exist
$DistDir = "dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}

# Copy the binary to dist with appropriate name
$BinaryName = if ($Target -eq "windows") {
    "rquest_runner.exe"
} else {
    "rquest_runner"
}

$SourcePath = "target/$RustTarget/release/$BinaryName"
$DestPath = "$DistDir/$BinaryName"

Write-Host "Copying binary to $DestPath..."
Copy-Item $SourcePath $DestPath -Force

Write-Host "`nBuild completed successfully!"
Write-Host "Binary location: $DestPath" 