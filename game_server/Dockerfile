FROM rust:latest AS builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libudev-dev \
    libasound2-dev \
    libwayland-dev \
    libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release --package game_server

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y \
    libasound2 \
    libwayland-client0 \
    libxkbcommon0 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/game_server /usr/local/bin/
CMD ["/usr/local/bin/game_server"]