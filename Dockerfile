# Build the demo application
FROM rust:latest AS builder-demo

WORKDIR /build/demo

COPY ./demo ./
RUN cargo build --release

# Build the server application
FROM rust:alpine AS builder-server

RUN apk update && apk add musl-dev
WORKDIR /build/server

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo build --release

# Run the server application
FROM alpine:latest

WORKDIR /app

COPY --from=builder-demo /build/demo/target/release/demo ./demo
COPY --from=builder-server /build/server/target/release/dynamic-preauth ./dynamic-preauth

EXPOSE 5800
CMD ["/app/dynamic-preauth"]