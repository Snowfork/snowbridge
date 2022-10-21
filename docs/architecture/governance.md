# Governance

As a common-good project, the bridge and its components will be exclusively governed by Polkadot's governance. Specifically, the [Gov2](https://polkadot.network/blog/gov2-polkadots-next-generation-of-decentralised-governance/) governance model that is being proposed for Polkadot.

This promotes decentralisation in the following ways:

* No power will be vested in centralised collectives or multisigs
* Snowfork and its employees will have no executive control over the bridge and its locked-up collateral
* Anyone can participate in governance, from normal users to elected members of the Polkadot fellowship

## Cross-chain Governance

Our bridge has a significant number of contracts on the Ethereum, and these contracts need to be able to evolve along with the parachain side. Cross-chain governance will control both configuration and code upgrades on the Ethereum side.

As a prime example, Polkadot and BEEFY consensus algorithms will change, and so we need to make sure the Ethereum side of the bridge remains compatible. Otherwise locked up collateral will not be redeemable.

Smart contract upgrades and configuration changes will be triggered by Polkadot governance through the use of cross-chain messaging secured by the bridge itself.

## Fallbacks

Obviously there are implications if governance messages cannot be delivered to Ethereum. It would mean the Ethereum side would remain static and non-upgradable, eventually lead to a loss of funds.

As a mitigation, we envision heartbeat signals periodically being sent to the Ethereum side. If these signals are not received for a certain length of time, then a fallback governance mechanism will activate.

The fallback governance method will be limited to upgrading the BEEFY light client. This should be enough to re-establish cross-chain governance signalling.

There are various options for fallback governance. We believe the most decentralized fallback is a DAO that uses a voting system similar to what [Compound](https://docs.compound.finance/v2/governance/) uses.

### BeefyDAO

The initial membership of the BeefyDAO will be seeded by Polkadot governance through cross-chain messaging.

It's expected that most of the initial members will be Polkadot fellows. Over time, the ranks will grow to include various other stakeholders in the ecosystem.

The BeefyDAO is allowed to perform a single operation:

* Update the BEEFY light client contract

Once secure cross-chain signalling is re-established, the DAO will deactivate. &#x20;





