#!/usr/bin/env bash
# Render the Homebrew formula for wss-mux at a given tag.
#
# Usage: render-homebrew-formula.sh <ref-name> <repo>
#
# Pulls the .sha256 files from the GitHub Release (created earlier in the
# release workflow) and emits a formula to stdout. Run inside the CI
# workflow after `github-release` has uploaded the tarballs.

set -euo pipefail

REF_NAME="${1:?ref name required (e.g., v0.5.1)}"
REPO="${2:?repo required (e.g., alternet-dev/wss-mux)}"
VERSION="${REF_NAME#v}"
BASE_URL="https://github.com/${REPO}/releases/download/${REF_NAME}"

fetch_sha() {
  local target="$1"
  local url="${BASE_URL}/wss-mux-${REF_NAME}-${target}.tar.gz.sha256"
  # The .sha256 file is `<sha>  <filename>`; emit just the sha.
  curl --fail --silent --show-error --location "$url" | awk '{print $1}'
}

# Intel-Mac (x86_64-apple-darwin) is intentionally not built — see the
# matrix comment in .github/workflows/release.yml. Intel Mac users on
# Homebrew will get a "no available formula" message; they can install
# via `cargo install --git https://github.com/${REPO}` from source.
SHA_DARWIN_ARM=$(fetch_sha aarch64-apple-darwin)
SHA_LINUX_X86=$(fetch_sha x86_64-unknown-linux-gnu)
SHA_LINUX_ARM=$(fetch_sha aarch64-unknown-linux-gnu)

cat <<EOF
class WssMux < Formula
  desc "WebSocket multiplexer for server-driven event fanout"
  homepage "https://github.com/${REPO}"
  version "${VERSION}"
  license "MIT OR Apache-2.0"

  on_macos do
    on_arm do
      url "${BASE_URL}/wss-mux-${REF_NAME}-aarch64-apple-darwin.tar.gz"
      sha256 "${SHA_DARWIN_ARM}"
    end
  end

  on_linux do
    on_intel do
      url "${BASE_URL}/wss-mux-${REF_NAME}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${SHA_LINUX_X86}"
    end
    on_arm do
      url "${BASE_URL}/wss-mux-${REF_NAME}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${SHA_LINUX_ARM}"
    end
  end

  def install
    bin.install "wss-mux"
    doc.install "README.md", "LICENSE-MIT", "LICENSE-APACHE"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/wss-mux --version")
  end
end
EOF
