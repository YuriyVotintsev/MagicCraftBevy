#!/usr/bin/env bash
# Build Magic Craft for the browser (WebGL2).
# Usage:
#   scripts/build_web.sh              # dev profile (faster build, larger wasm)
#   PROFILE=release scripts/build_web.sh
#   FEATURES=dev scripts/build_web.sh # pass --features dev
#   JOBS=4 scripts/build_web.sh       # override parallel rustc jobs
set -euo pipefail

cd "$(dirname "$0")/.."

PROFILE="${PROFILE:-dev}"
FEATURES="${FEATURES:-}"
# On phones bevy_reflect/bevy_ecs at opt-level=3 hit ~1.5GB peak per rustc;
# 8 parallel jobs easily OOMs 8-12GB devices. 2 is a safe sweet spot.
JOBS="${JOBS:-2}"
TARGET="wasm32-unknown-unknown"
CRATE="magic_craft_bevy"
OUT_DIR="web/pkg"

# Disable incremental for wasm: saves disk, often cuts memory, and incremental
# is largely useless for release-like wasm builds anyway.
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

FEATURE_FLAG=()
if [ -n "$FEATURES" ]; then
  FEATURE_FLAG=(--features "$FEATURES")
fi

CARGO_JOBS=(--jobs "$JOBS")

if [ "$PROFILE" = "release" ]; then
  echo ">> cargo build --release --target $TARGET -j $JOBS ${FEATURE_FLAG[*]}"
  cargo build --release --target "$TARGET" "${CARGO_JOBS[@]}" "${FEATURE_FLAG[@]}"
  WASM="target/$TARGET/release/$CRATE.wasm"
else
  echo ">> cargo build --target $TARGET -j $JOBS ${FEATURE_FLAG[*]}"
  cargo build --target "$TARGET" "${CARGO_JOBS[@]}" "${FEATURE_FLAG[@]}"
  WASM="target/$TARGET/debug/$CRATE.wasm"
fi

EXPECTED_WBG_VERSION="$(awk '
  $1=="name" && $3=="\"wasm-bindgen\"" { want=1; next }
  want && $1=="version" { gsub(/"/, "", $3); print $3; exit }
' Cargo.lock)"
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "error: wasm-bindgen not on PATH." >&2
  echo "       Install: cargo install -f wasm-bindgen-cli --version $EXPECTED_WBG_VERSION" >&2
  exit 1
fi
GOT_WBG_VERSION="$(wasm-bindgen --version | awk '{print $2}')"
if [ -n "$EXPECTED_WBG_VERSION" ] && [ "$GOT_WBG_VERSION" != "$EXPECTED_WBG_VERSION" ]; then
  echo "warning: wasm-bindgen CLI is $GOT_WBG_VERSION but Cargo.lock pins $EXPECTED_WBG_VERSION." >&2
  echo "         Runtime/CLI version mismatches usually fail at runtime." >&2
  echo "         Fix: cargo install -f wasm-bindgen-cli --version $EXPECTED_WBG_VERSION" >&2
fi

mkdir -p "$OUT_DIR"
echo ">> wasm-bindgen --target web --out-dir $OUT_DIR $WASM"
wasm-bindgen --target web --no-typescript --out-dir "$OUT_DIR" "$WASM"

# Expose assets to the web server without copying.
if [ ! -e web/assets ]; then
  ln -s ../assets web/assets
fi

echo
echo "Build OK. Serve with: scripts/serve_web.sh"
