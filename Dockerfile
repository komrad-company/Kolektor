# ==============================================================================
# Build stage — compile kolektor-api (Rust)
# ==============================================================================
FROM rust:1.94-slim AS builder

WORKDIR /app

RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config=1.8.1-4 \
 && rm -rf /var/lib/apt/lists/*

# Cache des dépendances : copy manifests + stub main puis build pour remplir ~/.cargo
COPY api/Cargo.toml api/Cargo.lock* api/rust-toolchain.toml ./
COPY api/crates/kolektor-api/Cargo.toml    crates/kolektor-api/Cargo.toml
COPY api/crates/kolektor-common/Cargo.toml crates/kolektor-common/Cargo.toml
COPY api/crates/kolektor-seed/Cargo.toml   crates/kolektor-seed/Cargo.toml
RUN mkdir -p crates/kolektor-api/src crates/kolektor-common/src crates/kolektor-seed/src \
 && echo 'fn main(){}' > crates/kolektor-api/src/main.rs \
 && echo ''           > crates/kolektor-common/src/lib.rs \
 && echo ''           > crates/kolektor-seed/src/lib.rs
RUN cargo build --release 2>/dev/null; \
    rm -f target/release/kolektor-api target/release/deps/kolektor*

# Build réel — le COPY des sources + touch force le rebuild des crates internes
COPY api/crates/ ./crates/
COPY api/migrations/ ./migrations/
RUN touch crates/kolektor-api/src/main.rs \
         crates/kolektor-common/src/lib.rs \
         crates/kolektor-seed/src/lib.rs \
 && cargo build --release

# ==============================================================================
# Runtime stage — Vector + binaire kolektor-api dans la même image
# ==============================================================================
FROM timberio/vector:0.54.0-debian

# Patch des CVEs OS — voir .hadolint.yaml pour la justification de DL3005.
RUN apt-get update \
 && apt-get upgrade -y \
 && rm -rf /var/lib/apt/lists/*

LABEL maintainer="Benoit Caillabet"
LABEL description="Vector.dev + kolektor-api REST backend"

COPY catalog/ /etc/vector/catalog/
COPY --from=builder /app/target/release/kolektor-api /usr/local/bin/kolektor-api
COPY --from=builder /app/migrations /etc/kolektor/migrations
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT []
CMD ["/usr/local/bin/kolektor-api", "--help"]
