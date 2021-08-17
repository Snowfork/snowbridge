#!/bin/bash

set -o errexit

artifacts_dir=$1
network=$2
deployment_dir=$artifacts_dir/$network
etherscan_api_key=$3

verify_contract() {
  echo Verifying contract $1
  echo -n "module.exports = " > $deployment_dir/args_temp.js
  echo -n `jq '.args' $filename` >> $deployment_dir/args_temp.js
  echo ";" >> $deployment_dir/args_temp.js
  cat $deployment_dir/args_temp.js
  npx hardhat verify --network $2 \
    --constructor-args $deployment_dir/args_temp.js \
    `jq -r '.address' $filename`
}

rm -rf $deployment_dir/args_temp.js

for filename in $deployment_dir/*.json; do
  rm -rf $deployment_dir/args_temp.js
  verify_contract $filename $network
done

rm -rf $deployment_dir/args_temp.js
