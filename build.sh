#!/bin/bash
set -e

# Default to linux if no target specified
TARGET=${1:-linux}

if [ "$TARGET" != "linux" ] && [ "$TARGET" != "windows" ]; then
    echo "Error: Target must be 'linux' or 'windows'"
    exit 1
fi

# Set features
FEATURES="cli,emulation,gzip,brotli,deflate,zstd,rquest/full"

# Set target based on OS
if [ "$TARGET" = "windows" ]; then
    RUST_TARGET="x86_64-pc-windows-msvc"
    BINARY_NAME="rquest_runner.exe"
else
    RUST_TARGET="x86_64-unknown-linux-gnu"
    BINARY_NAME="rquest_runner"
fi

# Install target if not already installed
echo "Installing target $RUST_TARGET..."
rustup target add $RUST_TARGET

# Build the release binary
echo "Building release binary for $RUST_TARGET..."
cargo build --bin rquest_runner --release --target $RUST_TARGET --features $FEATURES

# Create dist directory if it doesn't exist
mkdir -p dist

# Copy the binary to dist
echo "Copying binary to dist/$BINARY_NAME..."
cp "target/$RUST_TARGET/release/$BINARY_NAME" "dist/$BINARY_NAME"

# Make binary executable on Linux
if [ "$TARGET" = "linux" ]; then
    chmod +x "dist/$BINARY_NAME"
fi

echo -e "\nBuild completed successfully!"
echo "Binary location: dist/$BINARY_NAME" 