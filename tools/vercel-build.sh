#!/usr/bin/env bash
# tools/vercel-build.sh — full build for a Vercel deployment of the frees web app.
#
# Referenced from /vercel.json as the `buildCommand` (framework preset "Other",
# outputDirectory web/dist). Vercel's build container has Node but NO Rust
# toolchain, so this script bootstraps one, then runs the two-stage build:
#
#   1. rustup (honours /rust-toolchain.toml: stable + wasm32-unknown-unknown)
#   2. wasm-pack v0.13.1 (the CI-pinned version) from the prebuilt release
#      tarball — `cargo install` fallback only if no prebuilt exists for the host
#   3. wasm-pack build crates/frees --release --target web
#         --out-dir ../../web/src/wasm/pkg     (the generated engine package)
#   4. cd web && npm run build                 (npm ci was the installCommand;
#                                               re-run here only if it is missing)
#
# The script is idempotent — every step checks before it acts, so re-running it
# locally on a machine that already has the toolchain skips straight to the
# builds — and it fails loudly: `set -euo pipefail` plus explicit existence
# checks on every artifact the next stage depends on.
#
# It works from any cwd (it cd's to the repo root) and is safe to run locally:
#   bash tools/vercel-build.sh

set -euo pipefail

WASM_PACK_VERSION="0.13.1"

log() { printf '\n==> %s\n' "$*"; }
die() { printf '\nERROR (tools/vercel-build.sh): %s\n' "$*" >&2; exit 1; }

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"
[ -f Cargo.toml ] && [ -d crates/frees ] || die "repo root not found at $REPO_ROOT (expected Cargo.toml + crates/frees)"

# ---------------------------------------------------------------------------
# 0. Node — the web build needs Node 22+ (web/.nvmrc). Node 20 can compile the
#    bundle but cannot run the vitest suite; on Vercel set the project's
#    Node.js Version to 22.x (package.json "engines" is deliberately not used).
# ---------------------------------------------------------------------------
command -v node >/dev/null 2>&1 || die "node not found on PATH — on Vercel this should never happen; locally install Node 22 (see web/.nvmrc)"
NODE_MAJOR="$(node -p 'process.versions.node.split(".")[0]')"
if [ "$NODE_MAJOR" -lt 20 ]; then
  die "Node >= 22 required (web/.nvmrc); found $(node --version). On Vercel: Project Settings -> Build & Development Settings -> Node.js Version -> 22.x"
elif [ "$NODE_MAJOR" -lt 22 ]; then
  printf 'WARNING: Node %s < 22 — the production build usually works, but web/.nvmrc pins 22 and the test suite requires it. Prefer Node 22.x.\n' "$(node --version)" >&2
fi

# ---------------------------------------------------------------------------
# 1. Rust toolchain. rustup reads /rust-toolchain.toml (channel = stable,
#    targets = ["wasm32-unknown-unknown"]) so a plain `rustup toolchain install`
#    brings exactly what the wasm build needs.
# ---------------------------------------------------------------------------
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export PATH="$CARGO_HOME/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  log "Installing rustup (toolchain pinned by rust-toolchain.toml)"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain none \
    || die "rustup installation failed"
  command -v rustup >/dev/null 2>&1 || die "rustup installed but not on PATH ($CARGO_HOME/bin)"
fi

log "Ensuring the pinned toolchain + wasm32-unknown-unknown target"
rustup toolchain install >/dev/null 2>&1 || rustup toolchain install \
  || die "rustup could not install the toolchain pinned in rust-toolchain.toml"
# Belt and braces: rust-toolchain.toml already lists the target, but an
# externally-provisioned toolchain may lack it.
rustup target add wasm32-unknown-unknown || die "could not add wasm32-unknown-unknown target"
command -v cargo >/dev/null 2>&1 || die "cargo not on PATH after toolchain install"
log "Using $(rustc --version) / $(cargo --version)"

# ---------------------------------------------------------------------------
# 2. wasm-pack, pinned to the CI version. Prefer the prebuilt release binary
#    (seconds) over `cargo install` (minutes of compilation).
# ---------------------------------------------------------------------------
wasm_pack_ok() {
  command -v wasm-pack >/dev/null 2>&1 \
    && [ "$(wasm-pack --version 2>/dev/null)" = "wasm-pack ${WASM_PACK_VERSION}" ]
}

if wasm_pack_ok; then
  log "wasm-pack ${WASM_PACK_VERSION} already installed"
else
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)   WP_TRIPLE="x86_64-unknown-linux-musl" ;;
    Linux-aarch64)  WP_TRIPLE="aarch64-unknown-linux-musl" ;;
    Darwin-arm64)   WP_TRIPLE="aarch64-apple-darwin" ;;
    Darwin-x86_64)  WP_TRIPLE="x86_64-apple-darwin" ;;
    *)              WP_TRIPLE="" ;;
  esac
  if [ -n "$WP_TRIPLE" ]; then
    log "Installing wasm-pack ${WASM_PACK_VERSION} (prebuilt, ${WP_TRIPLE})"
    WP_TMP="$(mktemp -d)"
    trap 'rm -rf "$WP_TMP"' EXIT
    curl -sSfL "https://github.com/rustwasm/wasm-pack/releases/download/v${WASM_PACK_VERSION}/wasm-pack-v${WASM_PACK_VERSION}-${WP_TRIPLE}.tar.gz" \
      | tar -xz -C "$WP_TMP" \
      || die "download/extract of wasm-pack ${WASM_PACK_VERSION} failed"
    mkdir -p "$CARGO_HOME/bin"
    install -m 0755 "$WP_TMP"/wasm-pack-v${WASM_PACK_VERSION}-${WP_TRIPLE}/wasm-pack "$CARGO_HOME/bin/wasm-pack" \
      || die "could not install the wasm-pack binary into $CARGO_HOME/bin"
  else
    log "No prebuilt wasm-pack for $(uname -s)/$(uname -m) — falling back to cargo install (slow)"
    cargo install wasm-pack --version "${WASM_PACK_VERSION}" --locked \
      || die "cargo install wasm-pack ${WASM_PACK_VERSION} failed"
  fi
  wasm_pack_ok || die "wasm-pack ${WASM_PACK_VERSION} not functional after install (got: $(wasm-pack --version 2>&1 || true))"
fi

# ---------------------------------------------------------------------------
# 3. The wasm engine package. web/src/wasm/pkg is generated output (gitignored
#    and .vercelignored) — it must be built before the Vite build or the app
#    ships a UI with no engine (web/WASM-PORT.md).
# ---------------------------------------------------------------------------
log "Building the wasm engine (crates/frees -> web/src/wasm/pkg)"
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg \
  || die "wasm-pack build failed"
[ -f web/src/wasm/pkg/frees_bg.wasm ] && [ -f web/src/wasm/pkg/frees.js ] \
  || die "wasm-pack reported success but web/src/wasm/pkg is incomplete"

# ---------------------------------------------------------------------------
# 4. The web app. npm ci normally ran as the vercel.json installCommand;
#    reinstall only if node_modules is absent (idempotence for local runs).
#    web/.npmrc (legacy-peer-deps) applies because we run inside web/.
# ---------------------------------------------------------------------------
cd web
if [ ! -d node_modules ]; then
  log "node_modules missing — running npm ci --ignore-scripts"
  npm ci --ignore-scripts || die "npm ci failed"
fi

log "Building the web app (vite -> web/dist)"
npm run build || die "npm run build failed"

[ -f dist/index.html ]           || die "build finished but dist/index.html is missing"
[ -f dist/sw.js ]                || die "build finished but dist/sw.js (the PWA service worker) is missing"
[ -f dist/manifest.webmanifest ] || die "build finished but dist/manifest.webmanifest is missing"
ls dist/assets/*.wasm >/dev/null 2>&1 \
  || die "build finished but no .wasm asset in dist/assets — the engine did not make it into the bundle"

log "Done. web/dist is ready ($(du -sh dist | cut -f1))."
