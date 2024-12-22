# Build the demo application
FROM rust:latest AS builder-demo

WORKDIR /build/demo

COPY ./demo ./
# RUN --mount=type=cache,id=s/dynamic-preauth-demo,target=/build/demo/target/ \
#     --mount=type=cache,id=s/dynamic-preauth-demo,target=/usr/local/cargo/git/db \
#     --mount=type=cache,id=s/dynamic-preauth-demo,target=/usr/local/cargo/registry \
RUN cargo build --release

# Build the server application
FROM rust:alpine AS builder-server

RUN apk update && apk add musl-dev
WORKDIR /build/server

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.lock ./
# --mount=type=cache,id=s/<service id>-<target path>,target=<target path>
# RUN --mount=type=cache,id=s/dynamic-preauth,target=/build/server/target/ \
#     --mount=type=cache,id=s/dynamic-preauth,target=/usr/local/cargo/git/db \
#     --mount=type=cache,id=s/dynamic-preauth,target=/usr/local/cargo/registry/ \
RUN cargo build --release

# Run the server application
FROM alpine:latest

WORKDIR /app

EXPOSE $PORT
COPY --from=builder-demo /build/demo/target/release/demo ./demo
COPY --from=builder-server /build/server/target/release/dynamic-preauth ./dynamic-preauth

CMD ["/app/dynamic-preauth"]