#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="$ROOT/release"
rm -rf "$OUT"
mkdir -p "$OUT"

echo "[1/2] Build .app bundle"
npm run tauri -- build --bundles app

APP_SRC="$ROOT/src-tauri/target/release/bundle/macos/Local AI Hub.app"
if [[ ! -d "$APP_SRC" ]]; then
  echo "Expected .app not found at: $APP_SRC"
  echo "If you changed productName, update scripts/release-macos.sh."
  exit 1
fi

cp -R "$APP_SRC" "$OUT/"
echo "Copied: $OUT/$(basename "$APP_SRC")"

echo "[2/2] Build .dmg bundle"
npm run tauri -- build --bundles dmg

DMG_SRC="$(ls -t "$ROOT"/src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -n 1 || true)"
if [[ -z "${DMG_SRC:-}" ]]; then
  echo "No .dmg found in src-tauri/target/release/bundle/dmg/"
  exit 1
fi

cp "$DMG_SRC" "$OUT/"
echo "Copied: $OUT/$(basename "$DMG_SRC")"

echo
echo "Done. Artifacts are in: $OUT"

