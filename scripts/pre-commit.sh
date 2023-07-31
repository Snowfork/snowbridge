#!/usr/bin/env bash

set -e

// checks that the cumulus submodule is in sync with our substrate dependencies
check_duplicate_versions() {
        local duplicates=`grep -Ei "source = \"git.*$1" cumulus/Cargo.lock parachain/Cargo.lock | sed 's/\.git//g' | sort -u`
        local version_count=`echo "$duplicates" | awk -F':' '{for (i=2; i<NF; i++) printf $i " "; print $NF}' | sort -u | wc -l | xargs`
        if [ "$version_count" != "1" ]; then
                echo Duplicate $1 versions detected
                echo "$duplicates"
                exit 1
        fi
}

# check typos
chronic typos .

# lint and format for core contracts and typescript codes
(cd web && chronic pnpm lint && pnpm format)

# lint and format for relayer codes
(cd relayer && chronic mage lint && chronic go fmt ./...)

check_duplicate_versions substrate
check_duplicate_versions polkadot
