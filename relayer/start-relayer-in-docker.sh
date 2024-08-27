#!/usr/bin/env bash
set -eu

source ../web/packages/test/scripts/set-env.sh

start_relayer()
{
    docker compose up -d
}

stop_relayer()
{
    docker compose down
}

build_image()
{
    docker build -f Dockerfile -t ghcr.io/snowfork/snowbridge-relay .
}

if [ -z "${from_start_services:-}" ]; then
    echo "start relayers only!"
    trap kill_all SIGINT SIGTERM EXIT
    start_relayer
    wait
fi
