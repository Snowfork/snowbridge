#!/usr/bin/env bash
set -eu

new_version=$1
echo "Set API versions to $new_version:"
declare -a files=$(find packages -type f -name package.json)
for file in "${files[@]}"; do
    echo "Updating $file"
    sed -i "s/\"version\": .*/\"version\": \"$new_version\",/g" $file
done
echo "Set API versions done."
