---
description: How to run relayers that are rewarded for relaying bridge messages
---

# Run Relayers

## Snowbridge Infra Suite Repository

This guide references Github repository [Snowfork/snowbridge-infra-suite](https://github.com/Snowfork/snowbridge-infra-suite) for example scripts and config files.&#x20;

Clone this repository on your server.

```
git clone https://github.com/Snowfork/snowbridge-infra-suite.git
```

## Relayer Binary

### Build Relayer

To build the relayer from source, follow these steps.

1. [Install Mage](https://github.com/magefile/mage?tab=readme-ov-file#installation)
2. Clone Snowbridge repository and build the relayer

```
git clone https://github.com/Snowfork/snowbridge.git
cd snowbridge/relayer
mage build
```

The relayer binary will be built at `build/snowbridge-relay`. Copy the binary over to the snowbridge-infra-suite directory

```
cp build/snowbridge ../../snowbridge-infra-suite
```

### Docker

TODO update docker release

```
docker pull ghcr.io/snowfork/snowbridge-relay:169e308
```

## Private Keys

You will need the follow private keys for the relayers:

1. Ethereum private keys for&#x20;
   1. the BEEFY relayer
   2. the Parachain relayer (x2 keys - one for the primary governance channel and one for the secondary governance channel)
2. Polkadot private keys for
   1. the Beacon relayer
   2. the Execution relayer

Add the private key files in directory `keys` in `snowbridge-infra-suite`. Create the following key files:

* beefy-relayer.key
* parachain-0-relayer.key (primary governance)
* parachain-1-relayer.key (secondary governance)
* beacon-relayer.key
* execution-relayer.key

These keys are used by the scripts.

## BEEFY Relayer

The BEEFY relayer relays BEEFY commitments to track consensus on Polkadot.

To run the BEEFY relayer, run script `/scripts/start-beefy-relayer.sh` in `snowbridge-infra-suite`.

It expects:

* A config file at `./config/beefy-relayer.json` Set the following values in the config file:
  * `BeefyClient`: The contract address where the BeefyClient is deployed.
  * `beefy-activation-block`: The beefy block to start syncing from.
  * `endpoint`: Both the Polkadot and Ethereum nodes to connect to.
  * `gas-limit`: The Ethereum gaslimit to limit the gas used per transaction.
* An Ethereum private key file at `./keys/beefy-relayer.key`

## Parachain Relayer

The Parachain relayer relays messages from Polkadot to Ethereum for a certain channel. You need to run a relayer per channel that you would like to support.

To run the Parachain relayer, run script `/scripts/start-parachain-relayer.sh 0` in `snowbridge-infra-suite`.

It expects:

* A config file at `./config/parachain-0-relayer.json` Set the following values in the config file:
  * `BeefyClient`: The contract address where the BeefyClient is deployed.
  * `Gateway`: The contract address where the Gateway contract is deployed.
  * `beefy-activation-block`: The beefy block to start syncing from.
  * `endpoint`: All the endpoints for the respective chains.
  * `channel-id:` The channel of the messages that will be relayed.
  * `gas-limit`: The Ethereum gaslimit to limit the gas used per transaction.
* An Ethereum private key file at `./keys/parachain-0-relayer.key`

## Beacon Relayer

The Beacon relayer relays consensus updates on Ethereum to Polkadot.&#x20;

To run the Beacon relayer, run script `/scripts/start-beacon-relayer.sh` in `snowbridge-infra-suite`.

It expects:

* A config file at `./config/parachain-0-relayer.json` Set the following values in the config file:
  * `endpoint`: Both the Polkadot and Ethereum nodes to connect to.
* A Polkadot private key file at `./keys/beacon-relayer.key`

## Execution Relayer

The Execution relayer relays messages from Ethereum to Polkadot for a certain channel. You need to run a relayer per channel that you would like to support.

To run the Beacon relayer, run script `/scripts/start-execution-relayer.sh` in `snowbridge-infra-suite`.

It expects:

* A config file at `./config/parachain-0-relayer.json` Set the following values in the config file:
  * `Gateway`: The contract address where the Gateway contract is deployed.
  * `endpoint`: Both the Polkadot and Ethereum nodes to connect to.
  * `channel-id:` The channel of the messages that will be relayed.
* A Polkadot private key file at `./keys/beacon-relayer.key`

## Service Files

To register the relayers as systemd services, use each service file under `snowbridge-infra-suite/services`. Each service can be registered with these steps:

```
sudo systemctl enable services/beacon-relay.service
sudo systemctl start beacon-relay.service
sudo systemctl status beacon-relay.service
```
