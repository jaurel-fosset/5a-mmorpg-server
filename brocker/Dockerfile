FROM rust:latest AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --package brocker --recipe-path recipe.json
COPY . .
RUN cargo build --release --package brocker

FROM debian:bookworm-slim
ARG DOCKER_GID=967
RUN groupadd -g ${DOCKER_GID} docker
COPY --from=builder /app/target/release/brocker /usr/local/bin/
CMD ["/usr/local/bin/brocker"]