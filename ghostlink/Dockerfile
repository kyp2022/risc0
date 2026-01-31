FROM rust:latest as builder

# Install system dependencies
# build-essential for gcc/g++ (often needed for crates)
# clang/llvm sometimes needed for bindgen
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

# Copy Cargo files to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY methods/Cargo.toml methods/
COPY host/Cargo.toml host/

# Create dummy sources to cache dependencies build
RUN mkdir -p methods/src methods/guest/src host/src
# Create dummy main files to allow cargo build to pass for deps
RUN echo 'fn main() {}' > methods/guest/src/main.rs
RUN echo 'fn main() {}' > host/src/main.rs
# Create a dummy lib.rs for methods if it's a lib (it's a package with "methods" name, usually lib)
RUN echo '' > methods/src/lib.rs

# Build dependencies (this layer is cached)
# We need to explicitly point to the workspace members if it's a workspace, 
# but based on file structure, it seems they are linked. 
# Let's try building the host specifically which should pull in methods.
# Note: "cargo risc0 build" might be needed for methods, but let's stick to standard cargo build for deps first
RUN cargo build --release --manifest-path host/Cargo.toml || true

# Now copy the actual source code
COPY . .

# Build the actual application
# We need to ensure the guest is built first or as part of the host build script
# The host build.rs typically handles guest buid via risc0-build.
# We need to make sure the guest build environment is ready.
ENV CC=clang
ENV CXX=clang++

# Build the host (which builds the guest)
RUN cargo build --release --manifest-path host/Cargo.toml

# Runtime stage
# We use a slightly smaller image for runtime, or same for simplicity with dynamically linked libs
FROM rust:latest

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/ghostlink-host ./ghostlink-host

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=3000

# Run the binary
CMD ["./ghostlink-host"]
