# Governance

As a common-good project, the bridge and its components will be exclusively governed by Polkadot's governance. Specifically, the [Gov2](https://polkadot.network/blog/gov2-polkadots-next-generation-of-decentralised-governance/) governance model that is being proposed for Polkadot.

This promotes decentralisation in the following ways:

* No power will be vested in centralised collectives or multisigs
* Snowfork and its employees will have no executive control over the bridge and its locked-up collateral
* Anyone can participate in governance, from normal users to elected members of the Polkadot fellowship

## Cross-chain Governance

Our bridge has a significant number of contracts on the Ethereum, and these contracts need to be able to evolve along with the parachain side. Cross-chain governance will control both configuration and code upgrades on the Ethereum side.

As a prime example, Polkadot and BEEFY consensus algorithms will change, and so we need to make sure the Ethereum side of the bridge remains compatible. Otherwise locked up collateral will not be redeemable.

Smart contract upgrades and configuration changes will be triggered by governance on Polkadot, through the use of cross-chain messaging secured by the bridge itself. In effect this means that there will be no governance authority on the Ethereum side. So no centralised multisigs or anything like that.

### Fallbacks

Obviously there are implications if governance messages cannot be delivered to Ethereum for any reason. It would mean the Ethereum side would remain static and non-upgradable, potentially leading to a loss of funds.

As a mitigation, we envision heartbeat messages periodically being sent to the Ethereum side. If these messages are not received or fail verification for a certain period of time, then a fallback governance mechanism will activate.

The fallback governance method will be likely be constrained in its scope of authority, being limited to upgrading a limited number of critical contracts. The BEEFY light client being one of them.&#x20;

Fundamentally there are really only three options for fallback governance, each with its own compromises and tradeoffs. Ultimately the choice of fallback will need to be made by the community during the evaluation of our treasury proposal.

#### Compound-style voting

Voting-based governance similar to what [Compound](https://docs.compound.finance/v2/governance/) uses. This would require voting power to be distributed among various stakeholders and various checks and balances to prevent subversion.

Through an on-chain mechanism, voting shares could initially be seeded by the Polkadot fellowship on the Polkadot side. Each fellowship member could supply an ethereum address to be given a share of voting power. Obviously this is still subject to sybil attacks, as there is no way to prove that fellowship members actually control the ethereum accounts that participate in voting.

Another option is to distribute voting power based on assets locked up as collateral on the Ethereum side. This would incentivize voters to keep the bridge operational so their collateral can continue to be redeemable. However since the bridge is designed to be general-purpose, there will be no blessed apps/assets. So this approach will likely not be viable in the long-term.

#### Non-upgradable contracts

One option is to just not have any fallbacks, and design the contracts to be non-upgradable.

This would likely require changes to the BEEFY protocol to enable some kind of protocol versioning and signaling of new light client implementations.

The risk is that the bridge gets bricked due to bugs not discovered during audits.

#### Multisig

This is probably the most contentious option. If the bridge is bricked/dormant for a minimum length of time, a multisig could be allowed to upgrade a limited number of critical contracts. The governance actions initiated by the multisig would also be subject to a 2 week time lock delay.
