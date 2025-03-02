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
    libssl1.1 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /dahlia

COPY --from=builder /dahlia/target/release/dahlia .

#copy static files needed at runtime
COPY --from=builder /dahlia/assets/ ./assets/ 
COPY --from=builder /dahlia/style/ ./style/

ENTRYPOINT ["./dahlia"]
EXPOSE 3000
