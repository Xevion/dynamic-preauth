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

# Development server with hot reload
dev:
    @echo "Starting development server..."
    cargo watch -x run

# Simple development run (no hot reload)
run:
    @echo "Starting server..."
    cargo run

# Build release
build:
    @echo "Building release..."
    cargo build --workspace --release

# Security audit
audit:
    @echo "Running security audit..."
    cargo audit

# Build Docker image
docker-build:
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

# Quick development check
quick: format lint
    @echo "Quick check completed!"
