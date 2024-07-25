#!/bin/bash

export PATH=$PATH:~/bin

set -xe

pushd relayers

beefy_relay_eth_key=$(cat beefy-relayer.key)

snowbridge-relay-westend run beefy \
	--config beefy-relay-westend.json \
	--ethereum.private-key $beefy_relay_eth_key

popd

