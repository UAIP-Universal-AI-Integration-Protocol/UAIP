# UAIP Hub - Production-Grade Multi-Stage Docker Build
# Optimized for minimal image size and security

# ============================================================================
# Stage 1: Build Environment
# ============================================================================
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user and group
RUN groupadd -r uaip && useradd -r -g uaip uaip

# Set working directory
WORKDIR /build

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/uaip-core/Cargo.toml ./crates/uaip-core/
COPY crates/uaip-auth/Cargo.toml ./crates/uaip-auth/
COPY crates/uaip-registry/Cargo.toml ./crates/uaip-registry/
COPY crates/uaip-router/Cargo.toml ./crates/uaip-router/
COPY crates/uaip-security/Cargo.toml ./crates/uaip-security/
COPY crates/uaip-orchestrator/Cargo.toml ./crates/uaip-orchestrator/
COPY crates/uaip-adapters/Cargo.toml ./crates/uaip-adapters/
COPY crates/uaip-hub/Cargo.toml ./crates/uaip-hub/

# Create dummy source files to cache dependencies
RUN mkdir -p crates/uaip-core/src \
    crates/uaip-auth/src \
    crates/uaip-registry/src \
    crates/uaip-router/src \
    crates/uaip-security/src \
    crates/uaip-orchestrator/src \
    crates/uaip-adapters/src \
    crates/uaip-hub/src

RUN echo "fn main() {}" > crates/uaip-hub/src/main.rs && \
    echo "// dummy" > crates/uaip-core/src/lib.rs && \
    echo "// dummy" > crates/uaip-auth/src/lib.rs && \
    echo "// dummy" > crates/uaip-registry/src/lib.rs && \
    echo "// dummy" > crates/uaip-router/src/lib.rs && \
    echo "// dummy" > crates/uaip-security/src/lib.rs && \
    echo "// dummy" > crates/uaip-orchestrator/src/lib.rs && \
    echo "// dummy" > crates/uaip-adapters/src/lib.rs && \
    echo "// dummy" > crates/uaip-hub/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin uaip-hub

# Remove dummy source files
RUN rm -rf crates/*/src

# Copy real source code
COPY crates ./crates

# Build actual application with release optimizations
RUN cargo build --release --bin uaip-hub

# Strip debug symbols to reduce binary size
RUN strip /build/target/release/uaip-hub

# ============================================================================
# Stage 2: Runtime Environment
# ============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user and group
RUN groupadd -r uaip && useradd -r -g uaip -s /sbin/nologin uaip

# Create directories
RUN mkdir -p /app/config /app/logs && \
    chown -R uaip:uaip /app

# Copy binary from builder
COPY --from=builder --chown=uaip:uaip /build/target/release/uaip-hub /app/uaip-hub

# Copy configuration files
COPY --chown=uaip:uaip config/ /app/config/

# Switch to non-root user
USER uaip

# Set working directory
WORKDIR /app

# Expose ports
EXPOSE 8443

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/uaip-hub", "health"] || exit 1

# Environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1

# Run the application
ENTRYPOINT ["/app/uaip-hub"]

# Metadata
LABEL org.opencontainers.image.title="UAIP Hub" \
      org.opencontainers.image.description="Universal AI Integration Protocol Hub" \
      org.opencontainers.image.vendor="Hakille" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP"
