### # Builder # ###
FROM rust:1 AS builder

# Files
WORKDIR /build
COPY . .

# Build
RUN cargo build --release

### # Runner # ###
FROM debian:stable-slim AS runner

# Packages
RUN apt-get update && \
    apt-get install -y \
    openssl ca-certificates

# Files
WORKDIR /app
COPY --from=builder /build/target/release/mapper .

# Execute
ENTRYPOINT ["./mapper"]