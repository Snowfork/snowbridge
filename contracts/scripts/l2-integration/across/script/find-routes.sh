#!/bin/bash

set -eux

# Mainnet API endpoints

## Fetch the list of supported chains on Across
curl -L https://app.across.to/api/swap/chains | jq .

## Fetch the list of supported tokens on Across
curl -L https://app.across.to/api/swap/tokens | jq .

## Fetch the available routes on Across
curl -L https://app.across.to/api/available-routes | jq .

# Testnet API endpoints

## Fetch the list of supported chains on Across
curl -L https://testnet.across.to/api/swap/chains | jq .

## Fetch the list of supported tokens on Across
curl -L https://testnet.across.to/api/swap/tokens | jq .

## Fetch the available routes on Across
curl -L https://testnet.across.to/api/available-routes | jq .
