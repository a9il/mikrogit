#!/usr/bin/env bash
# WebDriver E2E: launches the real app binary against a fixture repo and
# drives the UI through stage -> commit -> explorer -> merge editor -> terminal.
# Requires: tauri-driver (cargo install tauri-driver) plus
#   Linux:   WebKitWebDriver  (sudo apt install webkit2gtk-driver)
#   Windows: msedgedriver on PATH matching the WebView2 Runtime version
#            (see docs/windows.md)
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) APP="$ROOT/src-tauri/target/release/mikrogit.exe" ;;
  *)                    APP="$ROOT/src-tauri/target/release/mikrogit" ;;
esac

[ -x "$APP" ] || { echo "app binary missing: $APP (run pnpm tauri build first)"; exit 1; }
command -v tauri-driver >/dev/null || { echo "tauri-driver missing (cargo install tauri-driver --locked)"; exit 1; }

PORT="${E2E_PORT:-4444}"
FIXTURE="$ROOT/e2e/fixture-repo"
export WEBKIT_FORCE_SANDBOX=0
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export MIKROGIT_REPO="$FIXTURE"

# --- fixture repo: merge conflict + modified file + untracked + subfolder -----
rm -rf "$FIXTURE"
mkdir -p "$FIXTURE"
git -C "$FIXTURE" init -q -b main
git -C "$FIXTURE" config user.email e2e@test
git -C "$FIXTURE" config user.name "E2E"
git -C "$FIXTURE" config core.autocrlf false
echo "line-one" > "$FIXTURE/README.md"
echo "shared" > "$FIXTURE/file.txt"
mkdir -p "$FIXTURE/src"
echo "console.log(1)" > "$FIXTURE/src/app.js"
git -C "$FIXTURE" add .
git -C "$FIXTURE" commit -qm "initial"
git -C "$FIXTURE" checkout -qb feat
printf "feat change\n" > "$FIXTURE/file.txt"
git -C "$FIXTURE" commit -qam "feat"
git -C "$FIXTURE" checkout -q main
printf "main change\n" > "$FIXTURE/file.txt"
git -C "$FIXTURE" commit -qam "main"
git -C "$FIXTURE" merge feat >/dev/null 2>&1 || true # expected conflict
echo "line-one
line-two edited" > "$FIXTURE/README.md"
echo "new untracked file" > "$FIXTURE/notes.txt"

cleanup() {
  [ -n "${DRIVER_PID:-}" ] && kill "$DRIVER_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "starting tauri-driver on :$PORT"
tauri-driver --port "$PORT" >/tmp/mikrogit-tauri-driver.log 2>&1 &
DRIVER_PID=$!
sleep 1

E2E_APP="$APP" E2E_PORT="$PORT" node e2e/app.e2e.js
echo "E2E passed."
