# syntax=docker/dockerfile:1.7

# --- Build stage ----------------------------------------------------------
FROM rust:1-bookworm AS builder

WORKDIR /build

# Copy the manifest plus the source trees Cargo references. The [[test]]
# entry in Cargo.toml points at tests/integration/main.rs, so the directory
# has to exist even though we don't build the test target.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release --bin wss-mux

# --- Runtime stage --------------------------------------------------------
# distroless cc gives us glibc + libgcc (ring's HMAC has assembly + C linked
# against glibc), no shell, no package manager, nonroot user.
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /build/target/release/wss-mux /wss-mux

EXPOSE 8080
ENTRYPOINT ["/wss-mux"]
