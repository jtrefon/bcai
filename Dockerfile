# BCAI Production Dockerfile
# Multi-stage build for optimal performance and security

#=============================================================================
# Stage 1: Build Environment
#=============================================================================
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY runtime/Cargo.toml ./runtime/

# Create dummy source files to build dependencies
RUN mkdir -p runtime/src runtime/examples \
    && echo "fn main() {}" > runtime/src/main.rs \
    && echo "pub fn main() {}" > runtime/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --manifest-path runtime/Cargo.toml
RUN rm -rf runtime/src

# Copy actual source code
COPY runtime/src ./runtime/src
COPY runtime/examples ./runtime/examples

# Build the application
RUN cargo build --release --manifest-path runtime/Cargo.toml

#=============================================================================
# Stage 2: Production Runtime
#=============================================================================
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r bcai && useradd -r -g bcai bcai

# Create application directories
RUN mkdir -p /app/bin /app/data /app/logs /app/config \
    && chown -R bcai:bcai /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/runtime /app/bin/bcai-node
COPY --from=builder /app/target/release/examples/* /app/bin/

# Copy configuration files
COPY docker/config/* /app/config/

# Set up proper permissions
RUN chmod +x /app/bin/* \
    && chown -R bcai:bcai /app

# Switch to non-root user
USER bcai
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD /app/bin/bcai-node --health-check || exit 1

# Expose ports
EXPOSE 4001 8080 9090

# Environment variables
ENV RUST_LOG=info
ENV BCAI_CONFIG_PATH=/app/config
ENV BCAI_DATA_PATH=/app/data
ENV BCAI_LOG_PATH=/app/logs

# Default command
CMD ["/app/bin/bcai-node", "--config", "/app/config/production.toml"] 