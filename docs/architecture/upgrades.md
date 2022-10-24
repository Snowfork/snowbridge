# Upgrades

The Polkadot side of our bridge is easily upgradable using forkless runtime upgrades. On the Ethereum side, it is more complicated, since smart contracts are immutable.

In general, the design for our upgradable smart contracts is quite simple, relying on versioning of immutable contracts. This stands in contrast with the [proxy pattern](https://docs.openzeppelin.com/contracts/4.x/api/proxy), which while being very popular does have [drawbacks](https://blog.trailofbits.com/2018/09/05/contract-upgrade-anti-patterns/) which could impact the viability and security of the bridge.

## BEEFY Light Client

There are various constraints and circular dependencies which make upgrading the client more complicated.

1. The client is upgraded using cross-chain governance&#x20;
2. Cross-chain governance relies on the client to be operational and bug-free
3. For non-compatible changes in BEEFY consensus, the client needs to be upgraded before the relay chain.
4. A client is only operational once it has been configured with the details of the current and next validator set.&#x20;

To untangle these problems, we have the following strategy:

* The ethereum side of the bridge can have more than one registered client, and each client supports a different version of BEEFY.
* If a registered client has critical bug that breaks the bridge, then the [BeefyDAO](governance.md#beefydao) is allowed to replace that client with one that works.
* Newly registered clients will be configured with the current and next validator set from the previously registered client that is being replaced.

## Apps

Some of our core [App](apps/) smart contracts have assets locked up as collateral, which complicates our immutable contract approach.

Our solution is to structure app contracts as follows:

1. An outer shell that implements business logic and communicates with channels and users
2. An inner vault in which collateral is held

Upgrading a core app is thus a matter of deploying a new outer shell and transferring ownership of the vault to it. Vault contracts will be purposely kept very simple so they won't need to be upgraded.

## Change Management

Non-emergency upgrades will follow a formal change management process, including testing and security auditing.

Polkadot governance will need to therefore verify that all these steps have been carried out as part of the upgrade proposals. Questions which should be asked:

1. What are the upgrade steps?
2. How was the upgrade tested?
3. Was the upgrade audited?
4. What is the worst-case outcome if the upgrade fails?
5. What are the steps for reverting the upgrade? Can it be reverted?
