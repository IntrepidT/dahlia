#Dockerfile

FROM rust:1.75-slim-bullseye AS builder
ENV SQLX_OFFLINE=true
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    gcc \
    g++ \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /dahlia

#Copy source code
COPY . .
#Build the application
RUN cargo build --release 

#Create a new stage with minimal image
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /dahlia

COPY --from=builder /dahlia/target/release/dahlia .

#Set any environment variables
ENV LEPTOS_OUTPUT_NAME=dahlia
ENV LEPTOS_SITE_ROOT=/dahlia
ENV LEPTOS_SITE_PKG_DIR=/dahlia


#copy static files needed at runtime
COPY --from=builder /dahlia/assets/ ./assets/ 
COPY --from=builder /dahlia/style/ ./style/

ENTRYPOINT ["./dahlia"]
EXPOSE 3000

