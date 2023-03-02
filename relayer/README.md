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

Compile the contracts in the [contracts](../core/packages/contracts) directory:

```bash
pnpm build
```

Generate the bindings in the [contracts](contracts/) directory:

```bash
go generate ./...
```

## SSZ Encodings

To generate the SSZ encodings:

```
git clone https://github.com/ferranbt/fastssz.git
go run sszgen/*.go --path ../snowbridge/relayer/relays/beacon/state/beacon.go --objs BeaconStateBellatrixMainnet,BeaconStateBellatrixMinimal,BlockRootsContainerMainnet,BlockRootsContainerMinimal,TransactionsRootContainer
```

## Configuration

Note: For local development and testing, we use our E2E test stack described [here](../core/packages/test/README.md). It automatically generates suitable relayer configurations for testing.

For an example configuration, please consult the [setup script](https://github.com/Snowfork/snowbridge/blob/main/core/packages/test/scripts/start-services.sh) for our local development stack. Specifically the `start_relayer` bash function.

## Tests

To run both unit and integration tests, start a local E2E test stack and run the following command:

```bash
mage test
```
