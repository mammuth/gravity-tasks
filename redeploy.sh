#!/usr/bin/env bash
set -euo pipefail

git pull

docker compose -f docker-compose.prod.yml down --remove-orphans
DOCKER_BUILDKIT=1 docker compose -f docker-compose.prod.yml build --parallel=false
docker compose -f docker-compose.prod.yml up -d
