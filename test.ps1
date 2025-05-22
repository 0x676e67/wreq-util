# Clean dist directory
Write-Host "Cleaning dist directory..."
if (Test-Path dist) {
    Remove-Item -Path dist -Recurse -Force
}
New-Item -ItemType Directory -Path dist | Out-Null

# Build the release binary
Write-Host "Building release binary..."
cargo build --bin rquest_runner --release --features "cli,emulation,gzip,brotli,deflate,zstd,rquest/full"

# Run the test
Write-Host "Running test..."
.\target\release\rquest_runner.exe -P Chrome136 -m get -u https://cloudflare.com/cdn-cgi/trace > dist\trace_output.txt

# Add timestamp
Add-Content -Path "dist\trace_output.txt" -Value "Test completed at $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"

# Display results
Write-Host "Test Results:"
Get-Content dist\trace_output.txt 