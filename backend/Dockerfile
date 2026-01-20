# Build stage
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/
COPY shared/Cargo.toml shared/
COPY wasm/Cargo.toml wasm/

# Create dummy files for dependency caching
RUN mkdir -p backend/src shared/src wasm/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "pub fn dummy() {}" > shared/src/lib.rs && \
    echo "pub fn dummy() {}" > wasm/src/lib.rs

# Build dependencies only
RUN cargo build --release --bin cqm-server 2>/dev/null || true

# Copy actual source code
COPY backend/src backend/src
COPY backend/migrations backend/migrations
COPY shared/src shared/src
COPY config config

# Touch main.rs to trigger rebuild
RUN touch backend/src/main.rs

# Build the application
RUN cargo build --release --bin cqm-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/cqm-server /app/cqm-server

# Copy config files
COPY config /app/config

# Create non-root user
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app

USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

EXPOSE 3000

ENV CQM_ENVIRONMENT=production
ENV RUST_LOG=info

CMD ["/app/cqm-server"]
