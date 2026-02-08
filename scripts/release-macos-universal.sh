#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="$ROOT/release"
rm -rf "$OUT"
mkdir -p "$OUT"

echo "[1/2] Build universal .app + .dmg"
npm run tauri -- build -t universal-apple-darwin -b app dmg

APP_SRC="$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/macos/Local AI Hub.app"
VERSION="$(node -e "const fs=require('fs'); console.log(JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8')).version)")"
DMG_SRC="$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/dmg/Local AI Hub_${VERSION}_universal.dmg"

if [[ ! -d "$APP_SRC" ]]; then
  echo "Expected .app not found at: $APP_SRC"
  exit 1
fi
if [[ ! -f "$DMG_SRC" ]]; then
  echo "Expected .dmg not found at: $DMG_SRC"
  exit 1
fi

echo "[2/2] Copy artifacts to release/"
cp "$DMG_SRC" "$OUT/"

APP_ZIP="$OUT/Local AI Hub_${VERSION}_universal.app.zip"
ditto -c -k --sequesterRsrc --keepParent "$APP_SRC" "$APP_ZIP"

echo
echo "Done. Artifacts are in: $OUT"
echo " - $(basename "$DMG_SRC")"
echo " - $(basename "$APP_ZIP")"
