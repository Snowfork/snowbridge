# Bridgerelayer

Relayer service that streams transactions from blockchain networks, packages data into messages, and sends the packets to the correlated bridge component.

Note: the bridgerelayer is currently in a boilerplate/architectural design state, it's not functional yet.

Thanks to Chainsafe for their work on [ChainBridge](https://github.com/ChainSafe/ChainBridge). This relayer service
is inspired by their design and incorporates some of their code.

## Development

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

## Configuration

Before running the relay, it needs to be configured first. Configuration is read from `~/.config/artemis-relay/config.toml` or the directory in which the relayer binary is located.

Here is an example config.toml:

```toml
[ethereum]
endpoint = "ws://localhost:9545/"

[ethereum.bridge]
address = "0x17f7C1e314180D8b8588CA50cF09A0e0847c77F6"
abi = "~/.config/artemis-relay/ethereum/Bridge.json"

[ethereum.apps]
[ethereum.apps.eth]
address = "0x95aF4D3B8938063486fE23C8D8867deD6aee5646"
abi = "~/.config/artemis-relay/ethereum/ETHApp.json"

[ethereum.apps.erc20]
address = "0xb664F267fa8775563E2aD1cED44a0996198F7eE0"
abi = "~/.config/artemis-relay/ethereum/ERC20App.json"

[substrate]
endpoint = "ws://127.0.0.1:9944/"
```

### Secrets

The relayer requires secret keys for submitting transactions to both chains. It reads these keys from the environment.

Example:

```
export ARTEMIS_ETHEREUM_KEY=603a72b0c0a65d9728353714d74291ea439c6816
export ARTEMIS_SUBSTRATE_KEY=//Alice
```

## Running the relay locally

For testing, start a local Ethereum network and deploy the Bank contract by following the set up instructions [here](../ethereum/README.md).

```bash
build/artemis-relay run
```

## Usage

```bash
# Check that the binary was successfully installed
artemis-relay --help

# Start the relayer
artemis-relay run
```

You should see a message similar to

```bash
INFO[0000] Connected to Ethereum chain ID 5777
INFO[0000] Subscribed to app 0xC4cE93a5699c68241fc2fB503Fb0f21724A624BB
```

You can send a `sendEth` transaction to the Bank contract with default values via the sendEth script located in polkadot-ethereum/ethereum/scripts/sendEth.js

```bash
# Send the transaction
truffle exec sendEth.js

# You should see the transaction in the bridgerelayer
INFO[0007] Witnessed tx 0x22c26a2d423bcc9622daba9410f5bdee1d047ec2e8be5c112a01b64224dbea5e on app 0xC4cE93a5699c68241fc2fB503Fb0f21724A624BB
```
