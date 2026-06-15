FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package fake_shard

FROM debian:trixie-slim
COPY --from=builder /app/target/release/fake_shard /usr/local/bin/
CMD ["fake_shard"]