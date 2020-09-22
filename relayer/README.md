# Relayer <!-- omit in toc -->

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packets to the correlated bridge component.

Thanks to Chainsafe for their work on [ChainBridge](https://github.com/ChainSafe/ChainBridge). This relayer service
is inspired by their design and incorporates some of their code.

- [Requirements](#requirements)
  - [Development](#development)
  - [Dependencies](#dependencies)
- [Configuration](#configuration)
  - [Secrets](#secrets)
- [Build](#build)
- [Run](#run)
- [Tests](#tests)

## Requirements

### Development

This project requires the following tools for day to day development:

* [Mage](https://magefile.org/): Used for build tasks
* [Revive](https://github.com/mgechev/revive): Used for linting instead of golint

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

### Dependencies

The relayer depends on the following:

* A running parachain
* An ethereum chain with our contracts deployed

Open a new terminal, and start the parachain
```
cd ../parachain
target/release/artemis-node --dev
```

To ensure the ethereum contracts are deployed, follow the [Setup](../ethereum/README.md#set-up) guide.

## Configuration

Before running the relayer, it needs to be configured first. By default the configuration file is read from  `~/.config/artemis-relay/config.toml`, but this can be overriden by passing the `--config PATH` flag to the relayer binary.

To autogenerate a valid config file, run:

```bash
scripts/make-config.sh > /tmp/relay-config.toml

# verify that the config looks like valid TOML
cat /tmp/relay-config.toml
```

Or, manually create a config file using the template below:
```toml
[ethereum]
endpoint = "ws://localhost:9545/"

[ethereum.bridge]
address = "0x17f7C1e314180D8b8588CA50cF09A0e0847c77F6"
abi = "/tmp/Bridge.json"

[ethereum.apps.eth]
address = "0x95aF4D3B8938063486fE23C8D8867deD6aee5646"
abi = "/tmp/ETHApp.json"

[ethereum.apps.erc20]
address = "0xb664F267fa8775563E2aD1cED44a0996198F7eE0"
abi = "/tmp/ERC20App.json"

[substrate]
endpoint = "ws://127.0.0.1:9944/"
```

### Secrets

The relayer requires secret keys for submitting transactions to both chains. It reads these keys from the environment.

Example:

```
export ARTEMIS_ETHEREUM_KEY=603a72b0c0a65d9728353714d74291ea439c6816
export ARTEMIS_SUBSTRATE_KEY=//Relay
```

## Build

```bash
mage build
```

## Run

Run the relayer with the config generated in [Configuration](#configuration).

```bash
build/artemis-relay run --config /tmp/relay-config.toml
```

## Tests

This will run both unit and integration tests. Please ensure that both the ethereum and substrate chains are running as described in [Service Dependencies](#service-dependencies).

```bash
mage test
```
