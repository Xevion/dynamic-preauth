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

# Development server with hot reload (backend + ensures frontend is built)
dev: frontend-build
    @echo "Starting backend development server with bacon..."
    @echo "Frontend is served from ./public (built from frontend/)"
    bacon run

# Watch backend only (for when frontend is already built)
dev-backend:
    @echo "Starting backend watch with bacon..."
    bacon run

# Watch and serve frontend only
dev-frontend:
    @echo "Starting frontend dev server..."
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
