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

buildah run $c mkdir -p /var/lib/snowbridge
buildah copy $c target/release/artemis /usr/local/bin/snowbridge

buildah config \
    --entrypoint '["/usr/local/bin/snowbridge"]' \
    --cmd '--base-path /var/lib/snowbridge' \
    --port 30333 \
    --port 9933 \
    --port 9944 \
    --volume /var/lib/snowbridge \
    --label org.opencontainers.image.title="Snowbridge Parachain Collator" \
    --label org.opencontainers.image.authors="Snowfork" \
    --label org.opencontainers.image.url="https://github.com/Snowfork/snowbridge" \
    --label org.opencontainers.image.source="https://github.com/Snowfork/snowbridge" \
    --label org.opencontainers.image.created="$(date --rfc-3339=seconds)" \
    --label org.opencontainers.image.version=${version} \
    --label org.opencontainers.image.revision=${revision} \
    $c

buildah commit $c snowbridge-collator:${version}

if [[ ${push} = true ]]; then
    buildah push localhost/snowbridge-collator:${version} ghcr.io/snowfork/snowbridge-collator:${version}
fi
