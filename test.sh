#!/bin/bash

# Build the release binary
echo "Building release binary..."
cargo build --bin rquest_runner --release --features "cli,emulation,gzip,brotli,deflate,zstd,rquest/full"

# Create dist directory
mkdir -p dist

# Run the test
echo "Running test..."
./target/release/rquest_runner -P Chrome136 -m get -u https://cloudflare.com/cdn-cgi/trace > dist/trace_output.txt

# Add timestamp
echo "Test completed at $(date '+%Y-%m-%d %H:%M:%S')" >> dist/trace_output.txt

# Display results
echo "Test Results:"
cat dist/trace_output.txt 