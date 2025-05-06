#!/usr/bin/env bash
set -eu

source ../web/packages/test/scripts/set-env.sh

build_image()
{
    docker build -f Dockerfile -t snowbridge-relay:local .
}

if [ -z "${from_start_services:-}" ]; then
    echo "Building local docker image"
    build_image
fi
