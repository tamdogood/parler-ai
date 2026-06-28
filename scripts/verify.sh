#!/usr/bin/env bash
#
# verify.sh — the single, reliable feedback signal for the autonomous loop.
#
# Loop engineering lives or dies on one thing: a fast, deterministic gate the agent can run after
# every change to learn "am I done / did I break something?". This script IS that gate. It mirrors
# `.github/workflows/ci.yml` exactly so "green locally" == "green in CI" — no surprises after a push.
#
# Contract: exits 0 and prints `VERIFY: PASS` only when every gate passes; otherwise exits non-zero
# and prints `VERIFY: FAIL (<stage>)`. The loop greps for those lines.
#
# IMPORTANT: never run `cargo fmt` here. This repo is hand-formatted; a repo-wide format would
# rewrite every file. (See tasks/lessons.md.)
#
# Usage:
#   scripts/verify.sh              # full gate: Rust (build · clippy · test) + web (build)
#   scripts/verify.sh --rust-only  # skip the web build (faster; use when you only touched crates)

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Match the Makefile: find cargo even when it isn't on PATH (common under launchd/cron).
CARGO="$(command -v cargo 2>/dev/null || echo "$HOME/.cargo/bin/cargo")"

# CI denies warnings; do the same so clippy/rustc warnings fail the gate locally too.
export RUSTFLAGS="${RUSTFLAGS:--D warnings}"
export CARGO_TERM_COLOR=always

RUST_ONLY=0
[ "${1:-}" = "--rust-only" ] && RUST_ONLY=1

fail() { echo "VERIFY: FAIL ($1)"; exit 1; }

echo "→ [1/4] cargo build --workspace --all-targets --locked"
"$CARGO" build --workspace --all-targets --locked || fail "rust-build"

echo "→ [2/4] cargo clippy --workspace --all-targets --locked -- -D warnings"
"$CARGO" clippy --workspace --all-targets --locked -- -D warnings || fail "clippy"

echo "→ [3/4] cargo test --workspace --locked"
"$CARGO" test --workspace --locked || fail "rust-test"

if [ "$RUST_ONLY" -eq 1 ]; then
  echo "→ [4/4] web build SKIPPED (--rust-only)"
else
  echo "→ [4/4] web: next build (type-checks every route)"
  (
    cd web
    # `next build` needs a COMPLETE dep tree. npm writes node_modules/.package-lock.json only on a
    # successful install, so use it as the "deps are good" sentinel: a missing-or-partial install
    # (e.g. one aborted by ENOSPC) lacks it and triggers a clean reinstall instead of failing later
    # on a half-unpacked module. Reruns with intact deps still skip straight to the build.
    [ -f node_modules/.package-lock.json ] || npm ci
    npm run build
  ) || fail "web-build"
fi

echo "VERIFY: PASS"
