FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package gatekeeper

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gatekeeper /usr/local/bin/
CMD ["/usr/local/bin/gatekeeper"]