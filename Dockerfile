# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
    apk add --no-cache bash curl nodejs pkgconfig openssl-dev musl-dev build-base npm clang libc-dev binaryen

RUN npm install -g sass

RUN cargo install sqlx-cli --no-default-features --features postgres

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY . .

RUN cargo leptos build --release -vv

FROM rustlang/rust:nightly-alpine as runner

# Add dependencies for Google Cloud SQL connectivity
RUN apk update && \
    apk add --no-cache ca-certificates libpq-dev curl

WORKDIR /app

COPY --from=builder /work/target/release/dahlia /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/assets /app/assets
COPY --from=builder /work/style/output /app/style/output
COPY --from=builder /work/Cargo.toml /app/
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=builder /work/migrations /app/migrations

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 3000

CMD ["/app/dahlia"]
