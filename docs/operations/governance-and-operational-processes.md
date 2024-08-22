# Governance and Operational Processes

## Introduction

The purpose of this document is to outline the governance structure and operational processes for Snowbridge. We aim to ensure that members of the fellowship understand and are comfortable with the proposed model, in case of an emergency, for the interventions to be whitelisted by the fellowship.&#x20;

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

For emergency situations, there are two possible scenarios:

### 1. Emergency Pause

In case of an emergency, a call to halt the bridge needs to be executed as soon as possible. Deploying the fix and resuming the bridge can happen afterwards with less time pressure and sensitivity, similar to a normal upgrade.

The processes to do so is:

{% code overflow="wrap" %}
```
git clone https://github.com/Snowfork/snowbridge.git
cd snowbridge/control

cargo run --bin snowbridge-preimage -- \
    --bridge-hub-api wss://polkadot-bridge-hub-rpc.polkadot.io \
    --asset-hub-api wss://polkadot-asset-hub-rpc.polkadot.io \
    halt-bridge \
    --all
```
{% endcode %}

The command will product a preimage hash, to be submitted to the **Whitelisted Caller Track**:

```
Preimage Hash: 0xc2569b432fba3b01df7da3c90bb546158480067064d4d5bc88c351fcba4355dd
Preimage Size: 129
0x1a0408630003000100a90f03242f00000602b28d0b9859460c530101200006028217b42..
```

The halt-bridge command has the following flags:

* `--all` Halt all the bridge components at once.
* `--ethereum-client` Halt the Ethereum client, blocking consensus updates.
* `--inbound-queue` Halt the Inbound Queue, blocking messages from BridgeHub to AssetHub.
* `--outbound-queue` Halt the Outbound Queue, blocking messages from AssetHub to BridgeHub.
* `--gateway` Halt messages from Ethereum to AssetHub.
* `--assethub-max-fee` Set the AssetHub to Ethereum to the maximum fee, so that users effectively cannot send messages from AssetHub to Ethereum.

Based on the nature of the emergency, the bridge might need to be halted in its entirety, or partially.

In case of emergency where there is uncertainty of the cause of a problem, it is best to block the bridge in its entirety using `halt-bridge --all`. To block both transfer directions at the earliest point possible, use `halt-bridge --gateway --assethub-max-fee`.

### 2. Emergency Upgrade

Although unlikely, there may be scenarios where the emergency can only be resolved through an upgrade rather than just a pause. In this case, the code for the upgrade may be sensitive, and we would want to avoid overly publicising it until the fix has been executed.

## Fallback governance

The Polkadot side of our bridge can be easily upgraded using forkless runtime upgrades. The process is more complex on the Ethereum side. The gateway contract on Ethereum consists of a proxy and an implementation contract. Polkadot governance can send a cross-chain message to the gateway, instructing it to upgrade to a new implementation contract.

For any emergencies that can be handled via Polkadot governance, the team aims to use a **Whitelisted Caller Track to fix any bugs**. This will allow the bridge to be updated in a speedy manner with the authorisation of Polkadot Fellowship (as both support and approval thresholds are lower than Root track) - we aim for the Fellowship members to ratify the use of Whitelisted Caller track for any emergency situation with Snowbridge: always taking into account an analysis on a case-by-case basis linked to each submission.&#x20;

On the Ethereum side, the design intentionally avoids fallback / backdoor governance mechanisms to maintain the bridge's integrity and security. Although there are early-stage ideas for fallback governance that don't involve backdoors, they are not likely to be implemented short term.

\
