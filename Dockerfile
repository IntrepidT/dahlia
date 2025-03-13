# Stage 1: Build the application
FROM ubuntu:24.04 as builder

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    nodejs \
    cargo \
    npm \
    clang \
    curl \
    bash \
    ca-certificates \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install tools
RUN npm install -g sass && \
    cargo install sqlx-cli --no-default-features --features postgres && \
    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh -o installer.sh && \
    chmod +x installer.sh && \
    ./installer.sh && \
    rm installer.sh && \
    rustup target add wasm32-unknown-unknown


WORKDIR /app


# Copy the rest of the source code
COPY . .

# Build the application
RUN cargo leptos build --release

# Stage 2: Create the runtime image
FROM ubuntu:24.04 as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 libc6 &&update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the build artifacts
COPY --from=builder /app/target/release/dahlia /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/assets /app/assets
COPY --from=builder /app/style/output /app/style/output
COPY --from=builder /app/Cargo.toml /app/
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations

#Ensure binary executable
RUN chmod +x /app/dahlia

# Set environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="./site"
ENV PATH="/usr/local/bin:$PATH"

# Expose the application port
EXPOSE 8080

# Run the application
CMD ["sh", "-c", "sqlx migrate run && /app/dahlia"]
