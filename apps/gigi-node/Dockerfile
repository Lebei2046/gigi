FROM rust:1.88-bookworm AS builder

WORKDIR /build

COPY Cargo.toml ./
COPY apps/gigi-node/ ./apps/gigi-node/

# Create minimal workspace Cargo.toml with dependencies
RUN sed -n '/^\[workspace\.dependencies\]/,/^$/p' Cargo.toml > workspace-deps.toml && \
    echo '[workspace]' > Cargo.toml && \
    echo 'resolver = "2"' >> Cargo.toml && \
    echo 'members = ["apps/gigi-node"]' >> Cargo.toml && \
    cat workspace-deps.toml >> Cargo.toml

# Generate new Cargo.lock for this minimal workspace
RUN cargo generate-lockfile

RUN cargo build --release --bin gigi-node --examples

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/gigi-node /usr/local/bin/gigi-node
COPY --from=builder /build/target/release/examples/group_messaging /usr/local/bin/group_messaging
COPY --from=builder /build/target/release/examples/docker_group_client /usr/local/bin/docker_group_client

RUN useradd -m -s /bin/bash gigi && \
    mkdir -p /data && \
    chown gigi:gigi /data

# Keep root user to ensure proper logging
# USER gigi
WORKDIR /app

ENTRYPOINT ["/usr/local/bin/gigi-node"]