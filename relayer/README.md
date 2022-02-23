# Relayer

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packages to the correlated bridge component.

- [Development](#development)
- [Configuration](#configuration)
  - [Secrets](#secrets)
- [Build](#build)
- [Run](#run)
- [Tests](#tests)

## Development

This project requires the following tools for day to day development:

- [Golang](https://go.dev/)
- [Mage](https://magefile.org/): Used for build tasks
- [Revive](https://github.com/mgechev/revive): Used for linting instead of golint

Please install them first.

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

The bindings in the [contracts](contracts/) directory for our Ethereum contracts are dynamically generated.

Make sure you have the following dependencies installed:

Install [jq](https://stedolan.github.io/jq/):

```bash
sudo apt install jq
```

Install [abigen](https://geth.ethereum.org/docs/dapp/native-bindings):

```
go install github.com/ethereum/go-ethereum/cmd/abigen@v1.10.6
```

Compile the contracts in the [ethereum](../ethereum) directory:

```bash
npx hardhat compile
```

Generate the bindings:

```bash
go generate ./...
```

## Configuration

Note: For local development and testing, we use our E2E test stack described [here](../test/README.md). It automatically generates suitable relayer configurations for testing.

For an example configuration, please consult the [setup script](https://github.com/Snowfork/snowbridge/blob/main/test/scripts/start-services.sh) for our local development stack. Specifically the `start_relayer` bash function.

## Tests

To run both unit and integration tests, run the following command:

```bash
mage test
```
