# --- ÉTAPE 1 : Préparation de l'environnement Chef ---
FROM rust:latest AS chef
RUN cargo install cargo-chef
RUN apt-get update && apt-get install -y \
    pkg-config \
    libudev-dev \
    libasound2-dev \
    libwayland-dev \
    libxkbcommon-dev \
    clang lld \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# --- ÉTAPE 2 : Planification des dépendances (Analyse du Workspace) ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- ÉTAPE 3 : Compilation (Le Builder) ---
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Utilisation du linker ultra-rapide LLD pour Bevy
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Cache BuildKit pour compiler Bevy et les dépendances externes (opt-level=3)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --package game_server --recipe-path recipe.json

# On copie l'intégralité des sources du workspace
COPY . .

# Cache BuildKit pour ton code applicatif (Profil dev rapide + incrémental)
# Le binaire est extrait du dossier 'target/debug' avant le démontage du cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --package game_server && \
    cp target/debug/game_server /app/game_server

# --- ÉTAPE 4 : Image Finale de Runtime ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libasound2 \
    libwayland-client0 \
    libxkbcommon0 \
    && rm -rf /var/lib/apt/lists/*

ARG DOCKER_GID=967
RUN groupadd -g ${DOCKER_GID} docker

# Récupération du binaire isolé à la racine du builder
COPY --from=builder /app/game_server /usr/local/bin/

CMD ["/usr/local/bin/game_server"]