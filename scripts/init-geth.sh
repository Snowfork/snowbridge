#!/usr/bin/env bash

set -eux
echo "Download geth for Mekong fork to replace the nix version"
git clone https://github.com/lightclient/go-ethereum.git
cd go-ethereum
make geth
mkdir -p $GOPATH/bin
cp geth $GOPATH/bin
