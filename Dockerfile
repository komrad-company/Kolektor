# ==============================================================================
# Build stage — compile kolektor-api (Rust)
# ==============================================================================
FROM rust:1.94-slim AS builder

WORKDIR /app

RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config \
 && rm -rf /var/lib/apt/lists/*

# Cache des dépendances : copy manifests + stub main puis build pour remplir ~/.cargo
COPY api/Cargo.toml api/Cargo.lock* api/rust-toolchain.toml ./
COPY api/crates/kolektor-api/Cargo.toml    crates/kolektor-api/Cargo.toml
COPY api/crates/kolektor-common/Cargo.toml crates/kolektor-common/Cargo.toml
COPY api/crates/kolektor-fetcher/Cargo.toml crates/kolektor-fetcher/Cargo.toml
COPY api/crates/kolektor-seed/Cargo.toml   crates/kolektor-seed/Cargo.toml
RUN mkdir -p crates/kolektor-api/src crates/kolektor-common/src crates/kolektor-fetcher/src crates/kolektor-seed/src \
 && echo 'fn main(){}' > crates/kolektor-api/src/main.rs \
 && echo 'fn main(){}' > crates/kolektor-fetcher/src/main.rs \
 && echo ''           > crates/kolektor-common/src/lib.rs \
 && echo ''           > crates/kolektor-seed/src/lib.rs
RUN cargo build --release 2>/dev/null; \
    rm -f target/release/kolektor-api target/release/kolektor-fetcher target/release/deps/kolektor*

# Build réel — le COPY des sources + touch force le rebuild des crates internes
COPY api/crates/ ./crates/
COPY api/migrations/ ./migrations/
RUN touch crates/kolektor-api/src/main.rs \
         crates/kolektor-fetcher/src/main.rs \
         crates/kolektor-common/src/lib.rs \
         crates/kolektor-seed/src/lib.rs \
 && cargo build --release

# ==============================================================================
# Runtime stage — Vector + binaire kolektor-api dans la même image
# ==============================================================================
FROM timberio/vector:0.54.0-debian

# Patch des CVEs OS (openssl, libc, dpkg...) — l'image Vector upstream n'est
# pas rebuild à chaque advisory Debian, donc on applique les security patches.
RUN apt-get update \
 && apt-get upgrade -y \
 && rm -rf /var/lib/apt/lists/*

LABEL maintainer="Benoit Caillabet"
LABEL description="Vector.dev + kolektor-api REST backend"

# Catalog de parsers (lu par `kolektor-api init` pour seeder la DB)
COPY catalog/ /etc/vector/catalog/

# Binaire Rust + migrations sqlx
COPY --from=builder /app/target/release/kolektor-api /usr/local/bin/kolektor-api
COPY --from=builder /app/target/release/kolektor-fetcher /usr/local/bin/kolektor-fetcher
COPY --from=builder /app/migrations /etc/kolektor/migrations

# Entrypoint legacy conservé en fallback pour deployments mono-source
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Vector sert uniquement de runtime ; l'entrypoint est choisi par le manifest K8s
# (kolektor-api init / serve / token) ou via /entrypoint.sh pour le legacy.
ENTRYPOINT []
CMD ["/usr/local/bin/kolektor-api", "--help"]
