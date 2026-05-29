#!/usr/bin/env bash
# Mint a HS256-signed JWT with the given principals for the e2e harness.
#
# Usage:
#   mint_token.sh <secret> <principal> [<principal> ...]
#
# Required tools: openssl, date. No jq or python — keeps the
# bootstrap surface tiny enough to run on a vanilla GH Actions runner
# or a developer's laptop without extra installs.

set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: $0 <secret> <principal> [<principal> ...]" >&2
  exit 2
fi

SECRET="$1"
shift

# Build the principals JSON array by hand so we don't pull in jq.
PRINCIPALS='['
SEP=''
for p in "$@"; do
  PRINCIPALS+="${SEP}\"${p}\""
  SEP=','
done
PRINCIPALS+=']'

NOW=$(date -u +%s)
EXP=$((NOW + 3600))  # 1 hour suffices for the harness; reconnect refresh
                     # is exercised separately in unit tests.

HEADER='{"alg":"HS256","typ":"JWT"}'
PAYLOAD="{\"iss\":\"e2e\",\"iat\":${NOW},\"exp\":${EXP},\"sub\":\"e2e-test\",\"principals\":${PRINCIPALS}}"

b64() {
  # base64url: + → -, / → _, strip padding.
  openssl base64 -e -A | tr '+/' '-_' | tr -d '='
}

ENC_HDR=$(printf '%s' "$HEADER" | b64)
ENC_PLD=$(printf '%s' "$PAYLOAD" | b64)
SIG=$(printf '%s.%s' "$ENC_HDR" "$ENC_PLD" \
  | openssl dgst -sha256 -mac HMAC -macopt "key:${SECRET}" -binary \
  | b64)

printf '%s.%s.%s' "$ENC_HDR" "$ENC_PLD" "$SIG"
