# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.86.0

# --- Chef Base Stage ---
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

# --- Demo Planner Stage ---
FROM chef AS demo-planner
COPY demo/Cargo.toml demo/Cargo.lock* demo/build.rs ./
COPY demo/src ./src
RUN cargo chef prepare --recipe-path recipe.json

# --- Demo Builder Stage ---
FROM chef AS demo-builder

# Install cross-compilation toolchain for Windows
RUN apt-get update && apt-get install -y \
    g++-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*

# Add cross-compilation targets
RUN rustup target add x86_64-pc-windows-gnu x86_64-unknown-linux-gnu

# Copy recipe and cook dependencies
COPY --from=demo-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-gnu --recipe-path recipe.json
RUN cargo chef cook --release --target x86_64-pc-windows-gnu --recipe-path recipe.json

# Copy source and build
COPY demo/Cargo.toml demo/Cargo.lock* demo/build.rs ./
COPY demo/src ./src

ARG RAILWAY_PUBLIC_DOMAIN
ENV RAILWAY_PUBLIC_DOMAIN=${RAILWAY_PUBLIC_DOMAIN}

RUN cargo build --release --target x86_64-unknown-linux-gnu
RUN cargo build --release --target x86_64-pc-windows-gnu

# Strip binaries
RUN strip target/x86_64-unknown-linux-gnu/release/demo

# --- Server Planner Stage ---
FROM chef AS server-planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# --- Server Builder Stage ---
FROM chef AS server-builder

# Copy recipe and cook dependencies
COPY --from=server-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Strip binary
RUN strip target/release/dynamic-preauth

# --- Frontend Builder Stage ---
FROM node:22-slim AS frontend-builder
WORKDIR /app

# Install pnpm
RUN corepack enable && corepack prepare pnpm@9 --activate

# Copy package files for layer caching
COPY frontend/package.json frontend/pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source and build
COPY frontend/ ./

ARG RAILWAY_PUBLIC_DOMAIN
ENV RAILWAY_PUBLIC_DOMAIN=${RAILWAY_PUBLIC_DOMAIN}

RUN pnpm build

# Pre-compress static assets
RUN ./compress.sh

# --- Runtime Stage ---
FROM debian:12-slim

ARG APP=/app
ARG APP_USER=appuser
ARG UID=1000
ARG GID=1000

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    wget \
    && rm -rf /var/lib/apt/lists/*

ARG TZ=Etc/UTC
ENV TZ=${TZ}

# Create non-root user
RUN addgroup --gid $GID $APP_USER \
    && adduser --uid $UID --disabled-password --gecos "" --ingroup $APP_USER $APP_USER \
    && mkdir -p ${APP}

WORKDIR ${APP}

# Copy built artifacts
COPY --from=frontend-builder --chown=$APP_USER:$APP_USER /app/dist/ ./public/
COPY --from=demo-builder --chown=$APP_USER:$APP_USER /app/target/x86_64-pc-windows-gnu/release/demo.exe ./demo.exe
COPY --from=demo-builder --chown=$APP_USER:$APP_USER /app/target/x86_64-unknown-linux-gnu/release/demo ./demo-linux
COPY --from=server-builder --chown=$APP_USER:$APP_USER /app/target/release/dynamic-preauth ./dynamic-preauth

# Set proper permissions
RUN chmod +x ${APP}/dynamic-preauth

USER $APP_USER

# Build-time arg for PORT, default to 5800
ARG PORT=5800
ENV PORT=${PORT}
EXPOSE ${PORT}

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT}/session || exit 1

CMD ["./dynamic-preauth"]
