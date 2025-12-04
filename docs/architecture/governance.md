# Governance

As a system bridge for Polkadot, it is exclusively governed by Polkadot's [OpenGov](https://polkadot.com/opengov/) governance model.

This promotes decentralisation in the following ways:

* No power is vested in centralised collectives or multisig accounts
* Snowfork and its employees have no control over the bridge and its locked-up collateral
* Anyone can participate in governance and vote on proposals.

## Cross-chain Governance

Our bridge has contracts on the Ethereum side, and these contracts need to be able to evolve along with the parachain side. Cross-chain governance will control both configuration and code upgrades on the Ethereum side.

As a prime example, Polkadot and BEEFY consensus algorithms will change, and so we need to make sure the Ethereum side of the bridge remains compatible. Otherwise locked up collateral will not be redeemable.

Smart contract upgrades and configuration changes are triggered by Polkadot governance through the use of cross-chain messaging secured by the bridge itself.

## Upgrades

The Polkadot side of our bridge is easily upgradable using forkless runtime upgrades. On the Ethereum side, it is more complicated, since smart contracts are immutable.

The gateway contract on Ethereum consists of a proxy and an implementation contract. Polkadot governance can send a cross-chain message to the Gateway, instructing it to upgrade to a new implementation contract.
