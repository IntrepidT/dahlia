# Get started with a build env with Rust nightly
FROM rust:1.79.0-alpine3.20 as builder

# If you’re using stable, use this instead
# FROM rust:1.74-bullseye as builder

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen && \
    npm install -g sass

# Install required tools
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.20/cargo-leptos-installer.sh | sh

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Build the app
RUN cargo leptos build --release -vv

FROM alpine:3.20.2 as runtime
WORKDIR /usr/bin
RUN apk add --no-cache ca-certificates tini wget

WORKDIR /app

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/dahlia ./

# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site ./site

# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml .

# Set any required env variables and
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"


# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["tini", "/app/dahlia"]