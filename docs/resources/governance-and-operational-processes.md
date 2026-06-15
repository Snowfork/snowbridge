# Governance and Operational Processes

## Introduction

The purpose of this document is to outline the governance structure and operational processes for Snowbridge. We aim to ensure that members of the fellowship understand and are comfortable with the proposed model, in case of an emergency, for the interventions to be whitelisted by the fellowship.

## Cross-chain Governance

Snowbridge is a common-good project, and its governance, including both configuration and code upgrades on the Ethereum side, will be exclusively managed by Polkadot's cross-chain governance system, secured by the bridge itself. This governance structure promotes decentralisation by:

1. Ensuring no power is vested in centralised collectives or multisig accounts.
2. Preventing Snowfork and its employees from having any control over the bridge or its locked-up collateral.
3. Allowing anyone, from regular users to elected members of the Polkadot fellowship, to participate in governance.

Polkadot's governance will oversee and trigger smart contract upgrades and configuration changes through cross-chain messaging, ensuring that the Ethereum side remains compatible with changes in Polkadot and BEEFY consensus algorithms.

## Governance API

The following calls are essential controls to maintain and operate the bridge effectively, and they must be initiated by the root origin via a suitable governance track, such as a whitelisted caller

* [upgrade](https://github.com/Snowfork/snowbridge/blob/c2142e41b5a2cbd3749a5fd8f22a95abf2b923d9/parachain/pallets/system/src/lib.rs#L304) - Upgrade the gateway contract
* [set\_operating\_mode](https://github.com/Snowfork/snowbridge/blob/c2142e41b5a2cbd3749a5fd8f22a95abf2b923d9/parachain/pallets/system/src/lib.rs#L332) - Set the operating mode of the gateway contract
* [set\_pricing\_parameters](https://github.com/Snowfork/snowbridge/blob/c2142e41b5a2cbd3749a5fd8f22a95abf2b923d9/parachain/pallets/system/src/lib.rs#L349) - Set fee/reward parameters

## Non-emergency Upgrades

We expect to need non-emergency governance calls once every few months as we improve the bridge and add new functionality. Fast ratification won't be as important for these calls, and they will be audited with public code to ensure transparency and security.

## Emergency Situations

Emergency response (halt-bridge and emergency-upgrade procedures) is documented in [Emergency Procedures](emergency-procedures.md). On-call operators should read that page directly.

## Fallback governance

The Polkadot side of our bridge can be easily upgraded using forkless runtime upgrades. The process is more complex on the Ethereum side. The gateway contract on Ethereum consists of a proxy and an implementation contract. Polkadot governance can send a cross-chain message to the gateway, instructing it to upgrade to a new implementation contract.

For any emergencies that can be handled via Polkadot governance, the team aims to use a **Whitelisted Caller Track to fix any bugs**. This will allow the bridge to be updated in a speedy manner with the authorisation of Polkadot Fellowship (as both support and approval thresholds are lower than Root track) - we aim for the Fellowship members to ratify the use of Whitelisted Caller track for any emergency situation with Snowbridge: always taking into account an analysis on a case-by-case basis linked to each submission.

On the Ethereum side, the design intentionally avoids fallback / backdoor governance mechanisms to maintain the bridge's integrity and security. Although there are early-stage ideas for fallback governance that don't involve backdoors, they are not likely to be implemented short term.
