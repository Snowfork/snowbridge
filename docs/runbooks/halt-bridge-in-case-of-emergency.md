---
description: >-
  In a case of an emergency (e.g. a exploitable vulnerability found), the bridge
  needs to be halted so that no messages can be processed.
---

# Halt Bridge in Case of Emergency

### Halting Mechanism

The bridge is halted through a storage item in each Snowbridge pallet called `OperatingMode` .

Possible operating mode states are:

* `Normal`
* `Halted` (`RejectingOutboundMessages` for the `system` pallet)

If the operating mode is set to [`Halted`](https://github.com/Snowfork/polkadot-sdk/blob/2536e780bf6af052e1d9e85a8b2648aae91ec6d7/bridges/snowbridge/primitives/core/src/operating\_mode.rs#L12), no bridge messages will be processed. Each pallet needs to be disabled individually. Here are the pallets that need to be disabled, with the call hash to do so:

* Ethereum client pallet: `0x520301`
* Inbound queue pallet: `0x500101`
* Outbound queue pallet: `0x510001`
* Ethereum system pallet: `0x530101`

These extrinsics should be done from the relay chain, descending to the BridgeHub parachain origin, similar to the [force beacon checkpoint call](https://app.gitbook.com/o/bDGMcdShFBeGc3v6VzHf/s/tC80IPpnYgEJmgOYIpqZ/\~/changes/72/runbooks/initialize-ethereum-light-client-with-forced-checkpoint).

If the bridge was halted, no messages will be processed. When the operating mode is changed to `normal` messages will be continue being processed.
