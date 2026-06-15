FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package brocker

FROM debian:trixie-slim
COPY --from=builder /app/target/release/brocker /usr/local/bin/
CMD ["broker"]