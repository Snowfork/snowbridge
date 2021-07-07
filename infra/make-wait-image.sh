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
    netcat \
	wget
buildah run $c apt-get autoremove -y
buildah run $c apt-get clean
buildah run $c find /var/lib/apt/lists/ -type f -not -name lock -delete

buildah copy --chmod 755 $c https://raw.githubusercontent.com/eficode/wait-for/v2.1.2/wait-for /usr/local/bin/wait-for

buildah config \
    --entrypoint '["/usr/local/bin/wait-for"]' \
    --label org.opencontainers.image.title="wait-for" \
    --label org.opencontainers.image.authors="Snowfork" \
    --label org.opencontainers.image.url="https://github.com/Snowfork/snowbridge" \
    --label org.opencontainers.image.source="https://github.com/Snowfork/snowbridge" \
    --label org.opencontainers.image.created="$(date --rfc-3339=seconds)" \
    --label org.opencontainers.image.version=${version} \
    --label org.opencontainers.image.revision=${revision} \
    $c

buildah commit $c wait-for:${version}

if [[ ${push} = true ]]; then
    buildah push localhost/wait-for:${version} ghcr.io/snowfork/wait-for:${version}
fi