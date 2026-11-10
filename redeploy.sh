#!/usr/bin/env bash
# Deploys the latest CI release (.github/workflows/release.yml) on the homelab CT.
set -euo pipefail

URL=https://github.com/mammuth/gravity-tasks/releases/latest/download/gravity-tasks-x86_64-linux-musl.tar.gz
APP_DIR=/opt/gravity-tasks
release="$APP_DIR/releases/$(date +%Y%m%d%H%M%S)"

mkdir -p "$release"
curl -fsSL "$URL" | tar -xz -C "$release" --strip-components=1
ln -sfn "$release" "$APP_DIR/current"
rc-service gravity-tasks restart

# Keep the last 3 releases for rollback (point `current` back at one and restart).
ls -1dt "$APP_DIR"/releases/* | tail -n +4 | xargs -r rm -rf
