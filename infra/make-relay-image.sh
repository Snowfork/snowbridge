#!/bin/bash

set -o errexit

push=false

while [[ $# -gt 0 ]]; do
    case "$1" in
    -p|--push)
        push=true
        shift
        ;;
    *)
        echo "error: unknown parameter"
        exit 1
        ;;
  esac
done

version=$(git describe --tags $(git rev-list --tags --max-count=1) | sed s/v//)
revision=$(git rev-parse HEAD)

c=$(buildah from ubuntu:20.04)

buildah run $c apt-get update
buildah run $c env DEBIAN_FRONTEND=noninteractive apt-get install -y \
    libssl1.1 \
	ca-certificates \
	curl
buildah run $c apt-get autoremove -y
buildah run $c apt-get clean
buildah run $c find /var/lib/apt/lists/ -type f -not -name lock -delete

buildah copy $c ../relayer/build/artemis-relay /usr/local/bin/snowbridge-relay

buildah config \
    --entrypoint '["/usr/local/bin/snowbridge-relay"]' \
    --cmd 'run --config /etc/snowbridge/relay.toml' \
    --volume /etc/snowbridge \
    --label org.opencontainers.image.title="Snowbridge Relay" \
    --label org.opencontainers.image.authors="Snowfork" \
    --label org.opencontainers.image.url="https://github.com/Snowfork/snowbridge" \
    --label org.opencontainers.image.source="https://github.com/Snowfork/snowbridge" \
    --label org.opencontainers.image.created="$(date --rfc-3339=seconds)" \
    --label org.opencontainers.image.version=${version} \
    --label org.opencontainers.image.revision=${revision} \
    $c

buildah commit $c snowbridge-relay:${version}

if [[ ${push} = true ]]; then
    buildah push localhost/snowbridge-relay:${version} ghcr.io/snowfork/snowbridge-relay:${version}
fi