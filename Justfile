# Justfile for dynamic-preauth
# Uses bacon for Rust watching, pnpm for frontend
# Frontend builds to ./public, which backend serves as static files

# Variables
image_name := "dynamic-preauth"
container_name := "dynamic-preauth-dev"
port := "5800"

# Default recipe
default:
    @just --list

# Run all checks (matches quality workflow)
check: format-check cargo-check lint audit frontend-check frontend-build
    @echo "All checks passed!"

# Format all Rust code
format:
    @echo "Formatting code..."
    cargo fmt --all

# Check formatting without modifying
format-check:
    @echo "Checking formatting..."
    cargo fmt --all -- --check

# Check code without building
cargo-check:
    @echo "Running cargo check..."
    cargo check --workspace --all-targets --all-features

# Lint with clippy
lint:
    @echo "Running clippy..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Frontend type check
frontend-check:
    @echo "Checking frontend..."
    pnpm --dir frontend astro check

# Build frontend
frontend-build:
    @echo "Building frontend..."
    pnpm --dir frontend build

# Build demo executables (debug mode for faster dev builds)
build-demo:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Building demo executables..."

    # Always build Linux demo
    echo "Building Linux demo..."
    cargo build --bin demo
    cp target/debug/demo ./demo-linux
    echo "  [OK] Linux demo built"

    # Try to build Windows demo if cross-compilation is available
    if rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
        echo "Building Windows demo..."
        if cargo build --bin demo --target x86_64-pc-windows-gnu 2>/dev/null; then
            cp target/x86_64-pc-windows-gnu/debug/demo.exe ./demo.exe
            echo "  [OK] Windows demo built"
        else
            echo "  [!] Windows build failed (mingw-w64 toolchain may not be installed)"
            echo "      Continuing without Windows demo..."
        fi
    else
        echo "  [SKIP] Windows target not installed"
        echo "         Install with: rustup target add x86_64-pc-windows-gnu"
        echo "         Also requires: sudo apt install mingw-w64"
    fi

    echo "Demo executables ready!"

# Development server with hot reload (backend + frontend using Overmind)
dev: build-demo
    @echo "Starting development servers with Overmind..."
    @echo ""
    @echo "Backend will run on:  http://localhost:5800"
    @echo "Frontend will run on: http://localhost:4321"
    @echo ""
    @echo "Overmind multiplexes logs with prefixes:"
    @echo "  [backend]  - Bacon watching Rust backend"
    @echo "  [frontend] - Astro dev server"
    @echo ""
    @echo "Overmind shortcuts:"
    @echo "  Ctrl+C     - Stop all processes"
    @echo "  'overmind connect <process>' - Attach to a specific process"
    @echo ""
    overmind start -f Procfile.dev

# Watch backend only (for when frontend is already built)
dev-backend: build-demo
    @echo "Starting backend watch with bacon..."
    bacon run

# Watch and serve frontend only
dev-frontend:
    @echo "Starting frontend dev server..."
    @echo "Make sure the backend is running on port 5800!"
    pnpm --dir frontend dev

# Simple development run (no hot reload)
run:
    @echo "Starting server..."
    cargo run --bin dynamic-preauth

# Build release
build:
    @echo "Building release..."
    cargo build --workspace --release

# Security audit
audit:
    @echo "Running security audit..."
    cargo audit

# Build Docker image (ensures frontend is built first)
docker-build: frontend-build
    @echo "Building Docker image..."
    docker build -t {{image_name}}:latest .

# Run Docker container
docker-run: docker-build
    @echo "Running Docker container..."
    docker run --rm -d --name {{container_name}} -p {{port}}:{{port}} -e PORT={{port}} {{image_name}}:latest
    @echo "Container started at http://localhost:{{port}}"

# Stop Docker container
docker-stop:
    @echo "Stopping Docker container..."
    docker stop {{container_name}} || true

# Docker logs
docker-logs:
    docker logs {{container_name}}

# Follow Docker logs
docker-logs-follow:
    docker logs -f {{container_name}}

# Clean Docker artifacts
docker-clean: docker-stop
    @echo "Cleaning Docker artifacts..."
    docker rmi {{image_name}}:latest || true

# Clean cargo artifacts
clean:
    @echo "Cleaning cargo artifacts..."
    cargo clean

# Full CI pipeline
ci: format-check lint frontend-check build docker-build
    @echo "CI pipeline completed!"

# Quick development check (format + clippy)
quick: format
    @echo "Running quick clippy check..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    @echo "Quick check completed!"

# Verify dev setup is ready (builds demo executables and checks dependencies)
smoke: build-demo
    @echo "Verifying development setup..."
    @echo ""
    @echo "Checking for overmind (required for 'just dev')..."
    @command -v overmind >/dev/null 2>&1 || { echo "  [!] overmind not found. Install from: https://github.com/DarthSim/overmind#installation"; exit 1; }
    @echo "  [OK] overmind found"
    @echo ""
    @echo "Checking for bacon..."
    @command -v bacon >/dev/null 2>&1 || { echo "  [!] bacon not found. Install with: cargo install bacon"; exit 1; }
    @echo "  [OK] bacon found"
    @echo ""
    @echo "Checking for pnpm..."
    @command -v pnpm >/dev/null 2>&1 || { echo "  [!] pnpm not found. Install from: https://pnpm.io/installation"; exit 1; }
    @echo "  [OK] pnpm found"
    @echo ""
    @echo "Checking demo executables..."
    @test -f ./demo-linux || { echo "  [!] demo-linux not found"; exit 1; }
    @echo "  [OK] demo-linux exists"
    @if [ -f ./demo.exe ]; then \
        echo "  [OK] demo.exe exists"; \
    else \
        echo "  [SKIP] demo.exe not found (Windows builds not available)"; \
    fi
    @echo ""
    @echo "[OK] Development setup is ready! Run 'just dev' to start."
