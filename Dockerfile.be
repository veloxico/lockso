# =============================================================================
# Lockso BE — Multi-stage multi-arch production build
# Supports: linux/amd64, linux/arm64
# Each platform builds natively via QEMU (no cross-compilation headaches).
# Result: ~20-30 MB distroless image
# =============================================================================

ARG RUST_VERSION=1.94

# ── Stage 1: Build (runs natively on each target platform via QEMU) ──
FROM rust:${RUST_VERSION}-bookworm AS builder

# Install OpenSSL dev headers for native build
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies — copy only Cargo files first
COPY backend/Cargo.toml backend/Cargo.lock* ./backend/
COPY backend/crates/lockso-api/Cargo.toml ./backend/crates/lockso-api/
COPY backend/crates/lockso-core/Cargo.toml ./backend/crates/lockso-core/
COPY backend/crates/lockso-crypto/Cargo.toml ./backend/crates/lockso-crypto/
COPY backend/crates/lockso-db/Cargo.toml ./backend/crates/lockso-db/

# Create dummy src files so cargo can resolve the workspace
RUN mkdir -p backend/crates/lockso-api/src && echo "fn main() {}" > backend/crates/lockso-api/src/main.rs \
    && mkdir -p backend/crates/lockso-core/src && echo "" > backend/crates/lockso-core/src/lib.rs \
    && mkdir -p backend/crates/lockso-crypto/src && echo "" > backend/crates/lockso-crypto/src/lib.rs \
    && mkdir -p backend/crates/lockso-db/src && echo "" > backend/crates/lockso-db/src/lib.rs

# Build dependencies only (cached layer)
RUN cd backend && cargo build --release 2>/dev/null || true

# Copy real source code + migrations
COPY backend/ ./backend/
COPY migrations/ ./migrations/

# Touch source files to invalidate the cache for actual code
RUN find backend/crates -name "*.rs" -exec touch {} +

# Build the real binary
RUN cd backend && cargo build --release --bin lockso && \
    cp target/release/lockso /app/lockso

# ── Stage 2: Runtime ──
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/lockso /usr/local/bin/lockso
COPY --from=builder /app/migrations /migrations

EXPOSE 8080

USER nonroot

ENTRYPOINT ["lockso"]
