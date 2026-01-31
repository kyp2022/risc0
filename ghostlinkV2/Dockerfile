FROM rust:latest as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    curl \
    git \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-binstall for faster tool installation
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install risc0 toolchain
RUN cargo binstall -y cargo-risc0
RUN cargo risc0 install

# Create app directory
WORKDIR /usr/src/app

# --- Dependency Caching Layer ---
# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY host/Cargo.toml host/
COPY methods/Cargo.toml methods/
COPY methods/guest/Cargo.toml methods/guest/

# Create dummy sources to allow dependency build
RUN mkdir -p host/src methods/src methods/guest/src
RUN echo 'fn main() {}' > host/src/main.rs
RUN echo 'fn main() {}' > methods/guest/src/main.rs
RUN echo '' > methods/src/lib.rs

# Build dependencies (release mode)
# This will compile all crate dependencies and cache them
RUN cargo build --release || true

# --- Application Build Layer ---
# Copy actual source code
COPY . .

# Force touch files to ensure cargo rebuilds them appropriate
# (cargo might think they are unchanged if timestamps are weird from copy)
RUN touch host/src/main.rs methods/src/lib.rs methods/guest/src/main.rs

# Set environment variables for build
ENV CC=clang
ENV CXX=clang++

# Build the workspace (release)
RUN cargo build --release --bin ghostlink-v2-host

# --- Runtime Layer ---
FROM rust:latest

WORKDIR /app

# Copy the binary from builder
# Note: The binary name is defined in host/Cargo.toml -> [package] name
COPY --from=builder /usr/src/app/target/release/ghostlink-v2-host ./ghostlink-service

# Expose port (Host maps 80->3000)
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=3000

# Run the binary
CMD ["./ghostlink-service"]
