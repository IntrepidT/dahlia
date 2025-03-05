# Build stage
FROM rustlang/rust:nightly-slim AS builder
ENV SQLX_OFFLINE=true

RUN apt-get update && apt-get install -y \
  build-essential \
  clang \
  curl \
  wget \
  libssl-dev \
  libpq-dev \
  pkg-config \
  sass

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

COPY . .

RUN cargo install cargo-leptos

RUN cargo leptos build --release

FROM debian:bookworm-slim as runner

RUN apt-get update && apt-get install -y \
  ca-certificates \
  libpq5 \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/dahlia /app/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/assets ./assets/
COPY --from=builder /app/style/ ./style/


ENV LEPTOS_OUTPUT_NAME="dahlia"
ENV APP_ENVIRONMENT="production"
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000" 

EXPOSE 3000

CMD ["/app/dahlia"]
