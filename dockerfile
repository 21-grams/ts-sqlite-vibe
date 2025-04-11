FROM rust:1.78 as builder

WORKDIR /usr/src/app
COPY . .

# Build the application with release optimizations
RUN cargo build --release

FROM debian:bookworm-slim

# Install OpenSSL and CA certificates (required for some Rust SSL functionality)
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/sensor-monitoring-api /app/sensor-monitoring-api

# Create a volume for the database
VOLUME ["/app/data"]

# Set environment variables
ENV DATABASE_PATH=/app/data/sensor_data.db
ENV RUST_LOG=sensor_monitoring_api=info,tower_http=info
ENV PORT=3000

# Expose the API port
EXPOSE 3000

# Run the binary
CMD ["/app/sensor-monitoring-api"]