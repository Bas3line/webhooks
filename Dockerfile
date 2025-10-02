FROM rust:1.75-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/webhook-service /usr/local/bin/webhook-service
COPY static/ ./static/

ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0:5050
ENV SQLX_OFFLINE=true

EXPOSE 5050

CMD ["webhook-service"]