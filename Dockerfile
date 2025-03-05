# syntax=docker/dockerfile:1

# Use the latest Rust alpine image for building
FROM rust:alpine3.20 AS build

# Install necessary build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl \
    pkgconfig \
    clang \
    lld \
    git \
    cargo \
    webkit2gtk-4.1-dev \
    curl \
    nodejs \
    npm \
    wasm-pack

# Set working directory
WORKDIR /app

# Install trunk for building Leptos frontend
RUN cargo install wasm-bindgen-cli trunk

# Copy project files
COPY . .

# Build the application
RUN trunk build --release

# Final stage
FROM alpine:3.20.6

# Install runtime dependencies
RUN apk add --no-cache \
    bzip2-dev \
    libgcc \
    libstdc++ \
    openssl \
    ca-certificates

# Create non-privileged user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    appuser

# Set user
USER appuser

# Copy built artifacts from build stage
# Note: Replace 'your-app-name' with your actual binary name from Cargo.toml
COPY --from=build /app/target/release/your-app-name /bin/server
COPY --from=build /app/dist /app/dist

# Expose port
EXPOSE 3000

# Run the application
CMD ["/bin/server"]