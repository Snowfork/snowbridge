---
description: >-
  How to set the Ethereum gateway controlling the bridge on the Ethereum side.
  This guide is specifically for setting the gateway address when deployed
  initially, not when the contracts are upgraded.
---

# Initial Deployment of Gateway Contracts

### Build and Deploy Contracts

To build and deploy the new contracts, follow these steps from inside the [Snowbridge repository](https://github.com/snowfork/snowbridge):

#### Generate BEEFY State

Set the following in .env file:

```bash
export RELAYCHAIN_ENDPOINT= <relay chain endpoint>
export BEEFY_START_BLOCK=  # default is 1
```

The BEEFY start block can be selected from a recent block that has a BEEFY commitment.

Run `scripts/generate-beefy-checkoutpoint.sh`. This will leave the beefy state in the contracts folder.

#### Deploy Contracts

Set the following in .env file:

```bash
# Secret
export INFURA_PROJECT_ID=<secret>
export ETHERSCAN_API_KEY=<secret>
export DEPLOYER_ETH_KEY=<secret>

# Chain
export ETH_NETWORK=< sepolia /mainnet >
export ETH_NETWORK_ID=< 11155111 /1 >

# Endpoints
export RELAYCHAIN_ENDPOINT=wss://rococo-rpc.polkadot.io
export ETH_RPC_ENDPOINT=https://sepolia.infura.io/v3
export ETH_WS_ENDPOINT=wss://sepolia.infura.io/ws/v3
export BEACON_HTTP_ENDPOINT=https://lodestar-sepolia.chainsafe.io

# Beefy
export BEEFY_START_BLOCK=8592280          # Default value
export MINIMUM_REQUIRED_SIGNATURES=16
export ETH_RANDAO_DELAY=128       # 4 epochs=128 slots=25.6mins
export ETH_RANDAO_EXP=6           # 6 slots before expired

# Channels and Agents
export BRIDGE_HUB_PARAID=1013
export BRIDGE_HUB_AGENT_ID=0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314
export ASSET_HUB_PARAID=1000
export ASSET_HUB_AGENT_ID=0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79
export REJECT_OUTBOUND_MESSAGES=false

# Fees
export CREATE_ASSET_FEE=10000000000
export DELIVERY_COST=10000000000
export EXCHANGE_RATE=25000000000000
export REGISTER_TOKEN_FEE=5000000000000000000
export RESERVE_TRANSFER_FEE=10000000000

export FOREIGN_TOKEN_DECIMALS=12

# Initial agents deposits. Set low on purpose as they can be topped up manually
export ETH_BRIDGE_HUB_INITIAL_DEPOSIT=1000000
```

Run `scripts/deploy-contracts.sh`

Back up `contracts.json` and contract artifacts to s3: [https://s3.console.aws.amazon.com/s3/buckets/snowbridge-rococo-demo?region=eu-central-1\&bucketType=general\&tab=objects](https://s3.console.aws.amazon.com/s3/buckets/snowbridge-rococo-demo?region=eu-central-1\&bucketType=general\&tab=objects)

Confirm settings with the team and get ETH for the deployment.&#x20;

Add contracts to tenderly

### Update the EthereumGatewayAddress on BridgeHub

To update the `EthereumGateway` [contract address](https://github.com/Snowfork/polkadot-sdk/blob/snowbridge/bridges/snowbridge/pallets/inbound-queue/src/lib.rs#L112), a set storage call is executed:

* Example call hash: `0xff00630003000100d50f03082f000006020700c817a804824f1200a400040440aed97c7854d601808b98ae43079dafb3505b4909ce6ca82d2ce23bd46738953c7959e710cd`
* Link to pre-populate on Rococo: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804824f1200a400040440aed97c7854d601808b98ae43079dafb3505b4909ce6ca82d2ce23bd46738953c7959e710cd](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804824f1200a400040440aed97c7854d601808b98ae43079dafb3505b4909ce6ca82d2ce23bd46738953c7959e710cd\))
