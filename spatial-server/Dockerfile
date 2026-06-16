FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package spatial-server

FROM debian:trixie-slim
COPY --from=builder /app/target/release/spatial-server /usr/local/bin/
CMD ["spatial-server"]
