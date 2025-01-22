# Relayer

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packages to the correlated bridge component.

- [Development](#development)
- [Configuration](#configuration)
  - [Secrets](#secrets)
- [Build](#build)
- [Run](#run)
- [Tests](#tests)

## Development

Run `mage` to see a list of available tasks (building, testing, linting, etc).

To enable revive for linting in VS-code, add the following to your config:

```json
{
    "go.lintTool": "revive",
    "go.lintFlags": [
        "-config=${workspaceFolder}/revive.toml"
    ],
}
```

## Contract Bindings

The relayer relies on dynamically generated bindings for our Ethereum contracts. They need to be updated whenever the contracts change.

Compile the contracts in the [contracts](../contracts) directory:

```bash
forge build
```

Generate contract bindings:

```bash
go generate ./...
```

## SSZ Encodings

To generate the SSZ encodings:

```
go install github.com/ferranbt/fastssz/sszgen
sszgen --path relays/beacon/state/beacon.go --objs BlockRootsContainerMainnet,TransactionsRootContainer,WithdrawalsRootContainerMainnet,BeaconStateDenebMainnet,BeaconBlockDenebMainnet,SignedBeaconBlockDeneb,SignedBeaconBlockElectra,BeaconStateElectra,BeaconBlockElectra
```

## Configuration

Note: For local development and testing, we use our E2E test stack described [here](../web/packages/test/README.md). It automatically generates suitable relayer configurations for testing.

For an example configuration, please consult the [setup script](https://github.com/Snowfork/snowbridge/blob/main/web/packages/test/scripts/start-services.sh) for our local development stack. Specifically the `start_relayer` bash function.


## Tests

To run both unit and integration tests, start a local E2E test stack and run the following command:

```bash
mage test
```

## Running

### Run message relayer with multiple instances

Configuration required for different relayers to coordinate. Take `execution-relay` which relayes message from Ethereum to AssetHub for example, assuming there are 3 instances deployed:


`execution-relay-asset-hub-0.json`

```

{
  ...
  "schedule": {
    "id": 0,
    "totalRelayerCount": 3,
    "sleepInterval": 20
  }
}

```

`execution-relay-asset-hub-1.json`

```

{
  ...
  "schedule": {
    "id": 1,
    "totalRelayerCount": 3,
    "sleepInterval": 20
  }
}

```

`execution-relay-asset-hub-2.json`

```

{
  ...
  "schedule": {
    "id": 2,
    "totalRelayerCount": 3,
    "sleepInterval": 20
  }
}

```

- id: ID of current relayer(start from 0)
- totalRelayerCount: Number of total count of all relayers
- sleepInterval: Sleep interval(in seconds) to check if message(nonce) has already been relayed

The configuration above applies also to multiple instances of `parachain-relay` which relayes message from AssetHub to Ethereum.


### Cost Analysis

#### Running Ethereum message relayer

For Ethereum message relayer take [extrinsic](https://bridgehub-polkadot.subscan.io/extrinsic/3264574-2) for example:

As we can see it will cost 0.041163771 DOT as transaction fee and the reward is 0.053783 DOT, so it's about 0.012 DOT as incentive for each message.

#### Running Parachain message relayer

For Parachain message relayer take [transaction]( https://dashboard.tenderly.co/snowfork/snowbridge-polkadot/tx/1/0x2dbcf28f8d80c43acd3f08e15b0ec2e3c2c8a929d50e0cba2e3bba5d39738bce) for example:

As we can see it will cost 0.000628 ETH as transaction fee and the reward is 0.000942 ETH, so it's about 0.0003 ETH as incentive for each message.
