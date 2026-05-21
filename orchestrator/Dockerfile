FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package orchestrator

FROM debian:bookworm-slim
ARG DOCKER_GID=967
RUN groupadd -g ${DOCKER_GID} docker
COPY --from=builder /app/target/release/orchestrator /usr/local/bin/
CMD ["/usr/local/bin/orchestrator"]