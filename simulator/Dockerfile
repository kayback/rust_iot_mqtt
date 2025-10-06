# Multi-stage build for optimized image size
FROM rust:slim-trixie as builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml ./
COPY ingestor ./ingestor
COPY simulator ./simulator

# Build simulator in release mode
WORKDIR /build/simulator
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/simulator /usr/local/bin/simulator

# Set environment
ENV RUST_LOG=info

CMD ["/usr/local/bin/simulator"]

