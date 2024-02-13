---
description: Processes for making changes to the Snowbridge and BridgeHub pallets
---

# Updating Snowbridge Pallets, BridgeHub & AssetHub Runtimes

## Writing New Code

Any new code will be added to the Snowfork repositories:

* [https://github.com/snowfork/snowbridge](https://github.com/snowfork/snowbridge) - For adding/modifying contracts, off-chain relayer code, test net setup scripts and smoke tests.
* [https://github.com/Snowfork/polkadot-sdk](https://github.com/Snowfork/polkadot-sdk) - For parachain and pallet changes.

## Merging New Code

#### Internal Snowfork Review

For both repositories mentioned above, pull requests (PR) should be made to the respective main branches. The Snowfork team members will review. Once the PR has been reviewed by 1 of more team members and all Github Actions pass, the pull request should be merged.

#### Parity Review

For any changes made to the [Snowfork/polkadot-sdk](https://github.com/Snowfork/polkadot-sdk), these changes should be contributed back to the original repository, [paritytech/polkadot-sdk](https://github.com/paritytech/polkadot-sdk).&#x20;

To create an upstream pull request, do the following steps:

1. Check out the [https://github.com/Snowfork/polkadot-sdk](https://github.com/Snowfork/polkadot-sdk) repository
2. Switch to the branch you would like to contribute upstream
3. Run \`./bridges/snowbridge/scripts/contribute-upstream.sh my-changes\`, where \`my-changes\` is the name of the new branch that will be created with your changes. The reason why this script creates a new branch is because we replaced Parity’s CI with our own, and so we need to clean up the changes that we have made to contribute the code back upstream. A new branch is created so it does not affect our CI and local development processes, but cleans the code so not to make irrelevant changes in the upstream PR.
4. Open the pull request on [paritytech/polkadot-sdk](https://github.com/paritytech/polkadot-sdk).
5. If the change is a minor change that doesn’t require release notes or greater awareness in Parity, ask on the PR in a comment for label R0-silent to be added to the PR. If the change is a larger change that requires awareness, add a file called \`pr\_xxx.prdoc\` in the \`prdoc\` directory, where xxx is the PR number. Describe the changes in the prdoc file (look at examples in that directory - it is fairly straightforward).
6. Usually, the Parity bridges team will review the PR within a day or two, without needing to prompt. For urgent reviews, post the link to the PR in the [Builders <> Snowfork Matrix Room](https://matrix.to/#/!gxqZwOyvhLstCgPJHO:matrix.parity.io?via=matrix.parity.io\&via=parity.io\&via=matrix.org) Builders <> Snowfork Matrix Room, asking for reviews.
7. If the change needs to be deployed to Rococo immediately (outside a regular release cycle), also update the relevant runtime spec version. This is typically the [BridgeHub](https://github.com/Snowfork/polkadot-sdk/blob/snowbridge/cumulus/parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/lib.rs#L206) or AssetHub runtime spec version. This spec version will automatically be incremented by Parity for release cycles.

### Crate Updates on crates.io

As part of the paritytech/polkadot-sdk release cycle, crates are published on crates.io. No extra action is required from the snowfork team to publish the Snowbridge crates. The crates are published by [parity-crate-owner](https://crates.io/users/parity-crate-owner).

### Auditing

Snowbridge pallets should be audited before releasing to Kusama and Polkadot. Audits should ideally be anticipated at least a month or two in advance, so that auditors can be engaged and booked in time. Since the overall codebase was audited, incremental audits will typically run for a week or less, with a week or two to address the findings, if necessary.

Audit fixes are usually done on a branch, so as not to interfere with other new features being built and to allow the auditors to easily verify fixes.

### Rococo Runtime Upgrade & Deployment Processes

Rococo deployments are done after the polkadot-sdk release. If out-of-cycle deployments need to be done, they can be arranged in [Chain Infrastructure: Rococo DevOps](https://matrix.to/#/!DiRwwDQntOGihlVwNO:parity.io?via=parity.io\&via=web3.foundation\&via=matrix.org).

### Kusama&#x20;

#### Runtime Upgrade

Once a new version of the polkadot-sdk is released, the polkadot-sdk crates should be updated in a PR to the fellowship runtimes repository. An example of such a PR is [Upgrade to latest polkadot-sdk@1.5 release #137](https://github.com/polkadot-fellows/runtimes/pull/137). Parity usually handles this and will push the release forward from the Fellows runtime PR to the execution of the upgrade on Polkadot.

#### Deployment

To deploy the change, extrinisc `parachainSystem.authorizeUpgrade` is called.

#### Voting

A proposal to upgrade the runtime is created and can be viewed on Polkassembly (e.g. [https://kusama.polkassembly.io/referendum/244](https://kusama.polkassembly.io/referendum/244))

#### Execution

Once the referendum receives enough votes, `parachainSystem.enactAuthorizedUpgrade` can be executed to enact the upgrade.

The above steps are handled by Parity devs.

### Polkadot

Polkadot is updated the same way as Kusama.

## Snowbridge Configuration

#### Ad-hoc Configuration Items

Configuration items to periodically configure for Snowbridge:

* **Set gateway:** Set the Ethereum gateway contract address.
  * call hash: `0xff00630003000100d50f03082f000006020700c817a804824f1200a400040440aed97c7854d601808b98ae43079dafb3505b4909ce6ca82d2ce23bd46738953c7959e710cd`
  * link to pre-populate on Rococo: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804824f1200a400040440aed97c7854d601808b98ae43079dafb3505b4909ce6ca82d2ce23bd46738953c7959e710cd](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804824f1200a400040440aed97c7854d601808b98ae43079dafb3505b4909ce6ca82d2ce23bd46738953c7959e710cd\))
* **Reset nonces:** Reset the nonces to 0. Only used if the bridge needs to be reset.
  * call hash: `0xff00630003000100d50f03082f000006020700c817a804824f1200a50200040821017d7c8b03a2a182824cfe569187a28faa718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000002101851a76c1adff357d59b36327d02cfb7f718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539200000000000000000`
  * link to pre-populate on Rococo: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804824f1200a50200040821017d7c8b03a2a182824cfe569187a28faa718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000002101851a76c1adff357d59b36327d02cfb7f718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539200000000000000000](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804824f1200a50200040821017d7c8b03a2a182824cfe569187a28faa718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a05392000000000000000002101851a76c1adff357d59b36327d02cfb7f718368a0ace36e2b1b8b6dbd7f8093c0594aa8a9c557dabac173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539200000000000000000\))
* **Enable Bridge:** Enable the bridge operating mode.
  * call hash: `0xff00630003000100d50f03082f000006020700c817a804c2d401000c530100`
  * link to pre-populate on Rococo: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804c2d401000c530100](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frococo-rpc.polkadot.io#/extrinsics/decode/0xff00630003000100d50f03082f000006020700c817a804c2d401000c530100\))

These items would require a referenda item to be created on OpenGov.

TBH Items:

* Which wallet will be used to create the referenda? Snowfork team members personal wallets?
* Exact steps to create referenda (e.g. extrinsic, renferenda on Polkassembly)
* Where is the voting and enactment periods set?
