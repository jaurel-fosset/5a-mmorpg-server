FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package orchestrator

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/orchestrator /usr/local/bin/
CMD ["/usr/local/bin/orchestrator"]