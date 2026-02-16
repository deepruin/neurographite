# Multi-stage Docker build for NeuroGraphite
# Stage 1: Rust builder
FROM rust:1.75 as rust-builder

# Install required packages
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src/ ./src/
COPY benches/ ./benches/

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user for security
RUN groupadd -r neurographite && useradd -r -g neurographite neurographite

# Create directories
RUN mkdir -p /app/data /app/frontend && \
    chown -R neurographite:neurographite /app

# Copy binary from builder
COPY --from=rust-builder /app/target/release/neurographite /app/neurographite

# Copy frontend assets
COPY frontend/ /app/frontend/

# Set ownership
RUN chown -R neurographite:neurographite /app

# Switch to non-root user
USER neurographite

# Set working directory
WORKDIR /app

# Expose port (default 8080)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV NEUROGRAPHITE_HOST=0.0.0.0
ENV NEUROGRAPHITE_PORT=8080
ENV NEUROGRAPHITE_DATA_DIR=/app/data

# Start the application
CMD ["./neurographite"]