# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.75.0
ARG APP_NAME=dahlia

# Build stage
FROM rust:${RUST_VERSION} AS build
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy project files
COPY . .

# Install cargo dependencies and build the application
RUN --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target/ \
    cargo build --release

# Final stage
FROM debian:bullseye-slim AS final

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

# Set user and working directory
USER appuser
WORKDIR /app

# Copy the built application from the build stage
COPY --from=build /app/target/release/${APP_NAME} /app/server

# Expose the application port
EXPOSE 3000

# Set the entrypoint
ENTRYPOINT ["/app/server"]