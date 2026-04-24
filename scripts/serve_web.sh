#!/usr/bin/env bash
# Serve the built web/ directory over HTTP for phone-browser testing.
# Binds to 0.0.0.0 so the phone can open it at the LAN IP of this host.
# Usage:
#   scripts/serve_web.sh              # port 8000
#   PORT=9000 scripts/serve_web.sh
set -euo pipefail

cd "$(dirname "$0")/../web"

PORT="${PORT:-8000}"
HOST="${HOST:-0.0.0.0}"

echo "Serving http://$HOST:$PORT  (open on phone browser)"
python3 -m http.server "$PORT" --bind "$HOST"
