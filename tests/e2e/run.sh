#!/usr/bin/env bash
# E2E harness: boots a real wss-mux server with a known manifest and
# runs both SDK e2e suites against it.
#
# Usage:
#   tests/e2e/run.sh                  # all suites
#   tests/e2e/run.sh rust             # rust only
#   tests/e2e/run.sh ts               # typescript only
#
# Requires: cargo, node ≥ 20, npm, openssl, curl.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HERE="$REPO_ROOT/tests/e2e"
PORT="${WSS_MUX_E2E_PORT:-18080}"
URL="ws://127.0.0.1:${PORT}/stream"
SSE_BASE="http://127.0.0.1:${PORT}"
HANDSHAKE_SECRET="e2e-handshake-secret-$$"
PUSH_TOKEN="e2e-push-secret-$$"
READ_TOKEN="e2e-read-secret-$$"
LOG="$HERE/server.log"

which_suite="${1:-all}"

mint() {
  "$HERE/mint_token.sh" "$HANDSHAKE_SECRET" "$@"
}

# Build the server. Use release so settle/RTT figures and CI runtime
# match what `cargo install` would produce.
echo ">> building wss-mux server (release)" >&2
cargo build --release --bin wss-mux --manifest-path "$REPO_ROOT/Cargo.toml" >&2

# Start the server.
echo ">> starting server on 127.0.0.1:${PORT}" >&2
WSS_MUX_LISTEN_ADDR="127.0.0.1:${PORT}" \
  WSS_MUX_PUSH_AUTH_TOKEN="$PUSH_TOKEN" \
  WSS_MUX_HANDSHAKE_SIGNING_KEY="$HANDSHAKE_SECRET" \
  WSS_MUX_STREAMS_MANIFEST_PATH="$HERE/manifest.yaml" \
  WSS_MUX_READ_AUTH_TOKEN="$READ_TOKEN" \
  "$REPO_ROOT/target/release/wss-mux" >"$LOG" 2>&1 &
SERVER_PID=$!

cleanup() {
  if kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# Wait for /health.
echo -n ">> waiting for /health" >&2
for _ in $(seq 1 60); do
  if curl -fsS "${SSE_BASE}/health" >/dev/null 2>&1; then
    echo " ready" >&2
    break
  fi
  echo -n "." >&2
  sleep 0.5
done
if ! curl -fsS "${SSE_BASE}/health" >/dev/null 2>&1; then
  echo
  echo "!! server failed to become ready; last 40 log lines:" >&2
  tail -n 40 "$LOG" >&2
  exit 1
fi

# Tokens.
MEMBER_TOKEN=$(mint role:member)
NOPRIV_TOKEN=$(mint role:guest)

FAIL=0

run_rust() {
  echo ">> rust SDK e2e" >&2
  WSS_MUX_E2E_URL="$URL" \
    WSS_MUX_E2E_TOKEN="$MEMBER_TOKEN" \
    WSS_MUX_E2E_NOPRIV_TOKEN="$NOPRIV_TOKEN" \
    WSS_MUX_E2E_PUSH_TOKEN="$PUSH_TOKEN" \
    WSS_MUX_E2E_PUSH_URL="${SSE_BASE}/events" \
    cargo test \
      --manifest-path "$REPO_ROOT/clients/rust/Cargo.toml" \
      --release \
      --test e2e \
      -- --include-ignored \
    || FAIL=1
}

run_ts() {
  echo ">> typescript SDK e2e" >&2
  (
    cd "$REPO_ROOT/clients/typescript"
    npm install --no-audit --no-fund >/dev/null
    npm run build >/dev/null
    WSS_MUX_E2E=1 \
      WSS_MUX_E2E_URL="$URL" \
      WSS_MUX_E2E_TOKEN="$MEMBER_TOKEN" \
      WSS_MUX_E2E_PUSH_TOKEN="$PUSH_TOKEN" \
      WSS_MUX_E2E_PUSH_URL="${SSE_BASE}/events" \
      npm run test:e2e
  ) || FAIL=1
}

run_cross() {
  # Cross-SDK matrix lives at the harness layer: each SDK's e2e covers
  # its own publish→subscribe roundtrip, and TS-publish→Rust-subscribe
  # (and the mirror) are asserted by the dedicated cross_sdk test in
  # the Rust e2e suite, which posts via HTTP and observes via WS so
  # neither side cheats by using its own in-memory queue.
  :
}

case "$which_suite" in
  rust) run_rust ;;
  ts)   run_ts ;;
  all)  run_rust ; run_ts ; run_cross ;;
  *)
    echo "unknown suite: $which_suite (rust|ts|all)" >&2
    exit 2
    ;;
esac

if [ "$FAIL" -ne 0 ]; then
  echo
  echo "!! one or more suites failed; last 40 server log lines:" >&2
  tail -n 40 "$LOG" >&2
  exit 1
fi

echo ">> ALL E2E PASSED"
