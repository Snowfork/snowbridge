---
description: Methods to keep up to date with Snowbridge dependency updates.
---

# Processes for keeping track of dependency changes

## Polkadot Changes

The [polkadot-sdk](https://github.com/paritytech/polkadot-sdk) is ultimately the place where any changes in the following components will be made. Snowbridge has several dependencies on components in the [polkadot-sdk](https://github.com/paritytech/polkadot-sdk):

* Polkadot's relay chain
* Bridge Hub parachain
* Substrate
* Cumulus

### Releases

The best way to get notifications about any changes in the polkadot-sdk, is to watch releases. See the last section on how to turn on notifications for [polkadot-sdk releases](https://github.com/paritytech/polkadot-sdk/releases).

### Polkadot-SDK tests

Snowbridge has a set of tests that test common bridge functionality, like register and sending a token from Ethereum to Polkadot, and to send the token back from Polkadot to Ethereum. These tests run on the polkadot-sdk CI, and so any incompatibility stemming from changes made in the polkadot-sdk will be caught by failing tests. These tests use the Rococo bridge hub and asset hub runtimes. It tests runtime configurations and Snowbridge pallets. The developer making the incompatible or breaking changes is responsible for making the fix as well. Noteworthy tests are:

* [Snowbridge emulated tests](https://github.com/paritytech/polkadot-sdk/blob/master/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs)
* [Snowbridge runtime tests](https://github.com/paritytech/polkadot-sdk/blob/master/cumulus/parachains/runtimes/bridge-hubs/bridge-hub-rococo/tests/snowbridge.rs)

### Smoke Tests

Apart from unit and emulated tests in the polkadot-sdk, Snowbridge also has a set of smoke tests, which uses a local testnet with a local Polkadot relay chain, asset hub and bridge hub parachains and Ethereum nodes. These smoke tests ultimately catch changes in the polkadot-sdk that may not have been caught by the polkadot-sdk tests.

These tests are run when a PR is merged into the polkadot-sdk repo - _<mark style="color:purple;">TODO we need to set this up</mark>_.

### Updating Snowfork Polkadot-SDK Fork

Since the Snowbridge team primarily works on a [fork of the polkadot-sdk](https://github.com/Snowfork/polkadot-sdk), the fork periodically needs to be updated from the [original repository](https://github.com/paritytech/polkadot-sdk). Pulling the latest code from paritytech/polkadot-sdk:master into snowfork/polkadot-sdk:snowbridge (the “snowbridge” branch is like our “main” branch) is another way to become aware of any changes. This update should be done bi-monthly, at the very least.



## Ethereum Changes

### Light Client Protocol

Ethereum consensus protocol changes are tracked on [https://github.com/ethereum/consensus-specs](https://github.com/ethereum/consensus-specs). Similarly to the polkadot-sdk, the release page should be watched for releases. Apart from this, on the main readme page of the Ethereum consensus repo page, any change stating “Light client sync protocol changes” should be followed:

### Ethereum Network Updates

Ethereum network updates are available at [https://blog.ethereum.org/category/protocol](https://blog.ethereum.org/category/protocol). At the bottom of the page is an email subscribe form signup. Sign up to get email notifications about protocol changes.&#x20;

### Lodestar

Snowbridge uses Lodestar as consensus node to relay headers to the on-chain light client. Lodestar should be kept up to date to support the latest Ethereum fork. Lodestar’s releases can be followed here:

[https://github.com/ChainSafe/lodestar/releases/](https://github.com/ChainSafe/lodestar/releases/tag/v1.15.0)

The release notes often contain the relevancy for a certain update, e.g. for v1.15.0: “This update is recommended to all users of Lodestar and mandatory for those running Sepolia and Holesky testnets. This release is also ready for the Gnosis Chain Chiado fork.”



## Enable Notifications for Releases on Github

To get notifications for Github repository releases, go to the Github repository, click on Watch -> Custom -> Releases.

Recommended repository releases to watch:

* [ethereum/consensus-specs](https://github.com/ethereum/consensus-specs)
* [ChainSafe/lodestar](https://github.com/ChainSafe/lodestar/releases/tag/v1.15.0)
* [paritytech/polkadot-sdk](https://github.com/paritytech/polkadot-sdk)

<figure><img src="https://lh7-us.googleusercontent.com/MsdyEhct1vKgHCgXWiFDvLg5DJ7CFPjSg52LpNNpATjmf0tzubFAI3Ti6nsAP2N5Rr8TdKlOnpmohObMXO9FJB6FFtSB2mqJ-Xdytq_BFxyTltpCxjex1PPJ793bXEqMbH7j5MlcjcB2zO1LAy_x2FI" alt=""><figcaption><p>Enable release notifications on Github</p></figcaption></figure>

## Update Notifications Checklist

* [ ] [ethereum/consensus-specs](https://github.com/ethereum/consensus-specs) Github releases
* [ ] [ChainSafe/lodestar](https://github.com/ChainSafe/lodestar/releases/tag/v1.15.0) Github releases
* [ ] [paritytech/polkadot-sdk](https://github.com/paritytech/polkadot-sdk) Github releases
* [ ] [Ethereum network update email notifications](https://blog.ethereum.org/category/protocol)
