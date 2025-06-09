# Stage 1: Build the application
FROM ubuntu:24.04 as builder

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    clang \
    curl \
    bash \
    ca-certificates \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y && \
    . $HOME/.cargo/env && \
    rustup target add wasm32-unknown-unknown

# Install other tools
RUN . $HOME/.cargo/env && \
    npm install -g sass && \
    cargo install sqlx-cli --no-default-features --features postgres && \
    cargo install --locked cargo-leptos

WORKDIR /app

# Copy only the files needed for dependency resolution first
COPY Cargo.toml Cargo.lock* ./

# Copy the rest of the source code
COPY . .

# Build the application
RUN . $HOME/.cargo/env && \
    cargo leptos build --release

# Stage 2: Create the runtime image
FROM ubuntu:24.04 as runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    curl \
    libc6 && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the build artifacts
COPY --from=builder /app/target/release/dahlia /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/assets /app/assets
COPY --from=builder /app/style/output /app/style/output
COPY --from=builder /app/Cargo.toml /app/
COPY --from=builder /root/.cargo/bin/sqlx /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/scripts /app/scripts

# Ensure binary is executable
RUN chmod +x /app/dahlia

# Set environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="./site"
ENV SMTP_SERVER=smtp.sendgrid.net 
ENV SMTP_PORT=587 
ENV SMTP_USERNAME=apikey 
ENV APP_URL=https://www.teapottesting.com 
ENV APP_ENV=production

# Expose the application port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["sh", "-c", "sqlx migrate run && /app/dahlia"]
