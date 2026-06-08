#!/bin/zsh
set -euo pipefail

SKILL_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SWIFTC_BIN="${SWIFTC_BIN:-$(xcrun --find swiftc)}"
SWIFT_SDK_PATH="${SWIFT_SDK_PATH:-$(xcrun --sdk macosx --show-sdk-path)}"

OUTPUT="${1:-/tmp/window.mp4}"
PID="${2:-}"

if [[ -z "$PID" ]]; then
  echo "usage: macos_window_record.sh <output.mp4> <pid>" >&2
  exit 1
fi

TMP_BIN="$(mktemp -u /tmp/macos_window_record-XXXXXX)"

"$SWIFTC_BIN" \
  -parse-as-library \
  -sdk "$SWIFT_SDK_PATH" \
  -target arm64-apple-macosx15.0 \
  -O \
  -framework ScreenCaptureKit \
  -framework AVFoundation \
  -framework CoreMedia \
  -framework AppKit \
  -framework Foundation \
  "$SKILL_ROOT/scripts/macos_window_record.swift" \
  -o "$TMP_BIN"

"$TMP_BIN" \
  --output "$OUTPUT" \
  --duration 5 \
  --pid "$PID" \
  --bundle-id "org.godotengine.godot" \
  --owner "Godot" \
  --title "Starship MMO" \
  --fps 30 \
  --wait 20
