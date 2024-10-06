# Stage 1: Build the Rust project
FROM rust:1.72 as builder

# Set the working directory inside the container
WORKDIR /app

# Install additional dependencies (like OpenSSL) required by some Rust crates (e.g., tokio, reqwest)
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy the Cargo.toml and Cargo.lock files to install dependencies
COPY Cargo.toml Cargo.lock ./

# Create an empty source file to cache dependencies installation
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Copy the actual source code
COPY ./src ./src

# Copy the .env file if it exists
COPY .env .env

# Build the project in release mode
RUN cargo build --release

# Stage 2: Create the final runtime container
FROM ubuntu:22.04

# Install necessary libraries (SSL and certificates)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    tzdata \
 && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/nepgram_bot /app/nepgram_bot

# Copy the .env file into the runtime container (if needed)
COPY --from=builder /app/.env .env

# Set environment variables for the Telegram bot (you may also add other required environment variables here)
ENV RUST_LOG=info

# Expose the port (if needed)
EXPOSE 8080

# Run the bot
CMD ["./nepgram_bot"]
