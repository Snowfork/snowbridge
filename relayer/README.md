# Relayer

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packets to the correlated bridge component.

Thanks to Chainsafe for their work on [ChainBridge](https://github.com/ChainSafe/ChainBridge). Our implementation is inspired by their design and incorporates some of their code.

- [Requirements](#requirements)
  - [Development](#development)
- [Configuration](#configuration)
  - [Secrets](#secrets)
- [Build](#build)
- [Run](#run)
- [Tests](#tests)

## Requirements

### Usage

For usage and development, you'll need:
- [Subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey): Used for substrate key management

### Development

This project requires the following tools for day to day development:

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

Make sure you have `jq` installed:

```bash
sudo apt install jq
```

Compile the contracts in another terminal window:

```bash
truffle compile --all
```

Generate the bindings:

```bash
go generate ./...
```

## Configuration

Before running the relayer, it needs to be configured first. By default the configuration file is read from  `~/.config/artemis-relay/config.toml`, but this can be overriden by passing the `--config PATH` flag to the relayer binary.

Example Configuration:

```toml
[ethereum]
endpoint = "ws://localhost:8545/"
descendants-until-final = 35

[ethereum.channels.basic]
inbound = "0x992B9df075935E522EC7950F37eC8557e86f6fdb"
outbound = "0x2ffA5ecdBe006d30397c7636d3e015EEE251369F"

[ethereum.channels.incentivized]
inbound = "0xFc97A6197dc90bef6bbEFD672742Ed75E9768553"
outbound = "0xEDa338E4dC46038493b885327842fD3E301CaB39"

[substrate]
endpoint = "ws://127.0.0.1:11144/"
```

NOTE: For development and testing, we use our E2E test stack described [here](../test/README.md). It automatically generates a suitable configuration for testing.

### Secrets

The relayer requires secret keys for submitting transactions to both chains. It reads these keys from environment variables.

Example:

```bash
export ARTEMIS_ETHEREUM_KEY=75fa57baca6ee656752e2daf522e75ded86d3ad24d660701aaa78e24b207f550
export ARTEMIS_SUBSTRATE_KEY=//Relay
```

## Build

```bash
mage build
```

## Run

Run the relayer with the configuration described in [Configuration](#configuration).

```bash
build/artemis-relay run --config config.toml
```

## Tests

To run both unit and integration tests, run the following command:

```bash
mage test
```
