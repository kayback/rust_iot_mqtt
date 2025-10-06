# Multi-stage build for optimized image size
FROM rust:slim-trixie as builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml ./
COPY ingestor ./ingestor
COPY simulator ./simulator

# Build ingestor in release mode
WORKDIR /build/ingestor
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/ingestor /usr/local/bin/ingestor

# Set environment
ENV RUST_LOG=info

# Expose HTTP port
EXPOSE 8080

CMD ["/usr/local/bin/ingestor"]

