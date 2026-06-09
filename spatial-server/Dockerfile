FROM rust:latest AS builder
WORKDIR /app
COPY src/ Cargo.toml .
RUN cargo build --release --package spatial_server

FROM debian:trixie-slim
COPY --from=builder /app/target/release/spatial_server /usr/local/bin/
CMD ["spatial_server"]
