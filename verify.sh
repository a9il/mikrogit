#!/usr/bin/env bash
# Full verification suite — run after every change set:
#   ./verify.sh            everything: tests, typecheck, E2E (auto-rebuilds the
#                          release binary if sources are newer than it)
#   ./verify.sh --fast     skip E2E / rebuild (tests + typecheck only)
#   ./verify.sh --watch    re-run the fast suite automatically on file changes
#   ./verify.sh --build    also rebuild the installable bundle afterwards
set -euo pipefail
cd "$(dirname "$0")"

FAST=0
WATCH=0
BUILD=0
for arg in "$@"; do
  case "$arg" in
    --fast) FAST=1 ;;
    --watch) WATCH=1 ;;
    --build) BUILD=1 ;;
    --e2e) ;; # E2E is on by default now; kept for compatibility
    *) echo "unknown option: $arg (usage: verify.sh [--fast] [--watch] [--build])"; exit 2 ;;
  esac
done

step() { printf '\n\033[1;34m=== %s ===\033[0m\n' "$1"; }

# Release binary name differs per platform.
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) BIN="src-tauri/target/release/mikrogit.exe" ;;
  *)                    BIN="src-tauri/target/release/mikrogit" ;;
esac

run_fast_suite() {
  step "Frontend tests (vitest)"
  pnpm test

  step "Typecheck (svelte-check)"
  pnpm check

  step "Rust unit tests"
  (cd src-tauri && cargo test --lib)

  step "Rust integration tests (real git repos)"
  (cd src-tauri && cargo test --test integration)
}

# --- watch mode: re-run the fast suite whenever sources change ----------------
if [ "$WATCH" -eq 1 ]; then
  STAMP="$PWD/.verify-stamp"
  touch "$STAMP"
  echo "watching src/, src-tauri/, e2e/ — Ctrl+C to stop"
  while true; do
    sleep 1
    changed=$(find src src-tauri/src src-tauri/tests src-tauri/Cargo.toml src-tauri/tauri.conf.json e2e \
      -not -path '*/fixture-repo/*' -type f -newer "$STAMP" 2>/dev/null | head -1)
    if [ -n "$changed" ]; then
      touch "$STAMP"
      echo -e "\n\033[1;33m--- change detected: $changed ---\033[0m"
      sleep 1 # let the editor finish writing
      bash "$0" --fast || true
      echo -e "\033[1;33m--- watching... ---\033[0m"
    fi
  done
fi

run_fast_suite

# --- E2E against the real app window -----------------------------------------
if [ "$FAST" -ne 1 ]; then
  need_rebuild=0
  [ -x "$BIN" ] || need_rebuild=1
  if [ "$need_rebuild" -eq 0 ] && [ -n "$(find src-tauri/src src-tauri/Cargo.toml src-tauri/build.rs src-tauri/tauri.conf.json \
      -type f -newer "$BIN" 2>/dev/null | head -1)" ]; then
    need_rebuild=1
  fi
  if [ "$need_rebuild" -eq 0 ] && [ -n "$(find src static svelte.config.js vite.config.js package.json \
      -type f -newer "$BIN" 2>/dev/null | head -1)" ]; then
    need_rebuild=1
  fi
  if [ "$need_rebuild" -eq 1 ]; then
    step "Rebuilding release binary (stale or missing)"
    pnpm tauri build >/dev/null 2>&1
  fi
  step "WebDriver E2E (real app window)"
  bash e2e/run.sh
fi

if [ "$BUILD" -eq 1 ]; then
  step "Rebuilding Tauri bundle"
  pnpm tauri build
fi

printf '\n\033[1;32mAll checks passed.\033[0m\n'
