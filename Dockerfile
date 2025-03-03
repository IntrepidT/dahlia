# Build stage
FROM rust:latest AS builder
ENV SQLX_OFFLINE=true

# Install build dependencies with cleanup in the same layer to reduce image size
RUN apt-get update && apt-get install -y --no-install-recommends clang \
    pkg-config libssl-dev libpq-dev gcc g++ \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-leptos \
    && rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy only files needed for dependency resolution first (better caching)
COPY Cargo.toml Cargo.lock ./

# Copy actual source code
COPY . .

# Build the application
RUN cargo leptos build --release && strip target/release/dahlia

# Runtime stage
FROM debian:bullseye-slim as runtime

# Create a non-root user to run the application
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev openssl libpq5 ca-certificates \
    && apt-get autoremove -y && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

# Copy binary and assets from builder stage
COPY --from=builder /app/target/release/dahlia /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/assets/ ./assets/
COPY --from=builder /app/style/ ./style/

# Set proper permissions
RUN chown -R appuser:appuser /app && \
    chmod -R 755 /app

# Switch to non-root user
USER appuser

# Environment variables
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000" \
    RUST_LOG="info" \
    LEPTOS_OUTPUT_NAME="dahlia" \
    LEPTOS_SITE_ROOT="site" \
    LEPTOS_SITE_PKG_DIR="/app"

# Expose port and set entrypoint
EXPOSE 3000
ENTRYPOINT ["./dahlia"]
