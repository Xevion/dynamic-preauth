# Build the demo application
FROM rust:latest AS builder-demo

WORKDIR /build/demo
RUN apt update && apt install -y g++-mingw-w64-x86-64
ARG RAILWAY_PUBLIC_DOMAIN

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-unknown-linux-gnu
# TODO: Add support for macOS
# RUN rustup target add x86_64-apple-darwin

COPY ./demo ./

RUN cargo build --release --target x86_64-pc-windows-gnu
RUN cargo build --release --target x86_64-unknown-linux-gnu
# RUN cargo build --release --target x86_64-apple-darwin

# Build the server application
FROM rust:alpine AS builder-server

RUN apk update && apk add musl-dev
WORKDIR /build/server

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo build --release

# Build the Astro frontend
FROM node:lts AS builder-astro

WORKDIR /build/astro

COPY ./frontend/ ./

ARG RAILWAY_PUBLIC_DOMAIN
ENV RAILWAY_PUBLIC_DOMAIN=$RAILWAY_PUBLIC_DOMAIN

RUN npm install pnpm -g
RUN pnpm install
RUN pnpm build
RUN ./compress.sh

# Run the server application
FROM alpine:latest

WORKDIR /app

COPY --from=builder-astro /build/astro/dist/ ./public/
COPY --from=builder-demo /build/demo/target/x86_64-pc-windows-gnu/release/demo.exe ./demo.exe
COPY --from=builder-demo /build/demo/target/x86_64-unknown-linux-gnu/release/demo ./demo-linux
COPY --from=builder-server /build/server/target/release/dynamic-preauth ./dynamic-preauth

EXPOSE 5800
CMD ["/app/dynamic-preauth"]