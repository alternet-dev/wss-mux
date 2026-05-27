# syntax=docker/dockerfile:1.7

# --- Build stage ----------------------------------------------------------
FROM rust:1-bookworm AS builder

WORKDIR /build

# Copy the manifest plus every source tree Cargo's manifest references.
# Cargo validates [[test]] / [[bench]] / [[bin]] target paths at manifest
# parse time — even though we only build --bin wss-mux, every declared
# target's source file must exist on disk or `cargo build` fails before
# compilation starts. So:
#   - tests/  — for [[test]] integration
#   - benches/ — for the [[bench]] dispatch/registry/envelope/cbor targets
#   - src/    — the actual build input, including src/bin/loadgen
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
COPY benches ./benches

RUN cargo build --release --bin wss-mux

# --- Runtime stage --------------------------------------------------------
# distroless cc gives us glibc + libgcc (ring's HMAC has assembly + C linked
# against glibc), no shell, no package manager, nonroot user.
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /build/target/release/wss-mux /wss-mux

EXPOSE 8080
ENTRYPOINT ["/wss-mux"]
