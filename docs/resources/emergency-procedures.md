# Emergency Procedures

This page is the on-call runbook for Snowbridge emergency response. It covers the two emergency scenarios: pausing the bridge (a non-destructive operating-mode flip) and emergency upgrades. For routine governance and the broader governance model, see [Governance and Operational Processes](governance-and-operational-processes.md).

For emergency situations, there are two possible scenarios.

## Producing a halt-bridge preimage

There are two interchangeable ways to produce the halt-bridge preimage. Both wrap the same underlying halt SDK and emit identical preimage bytes, so pick whichever is faster to reach during an incident.

* **Governance page (recommended)**: browser UI at [app.snowbridge.network/governance](https://app.snowbridge.network/governance). Select the halt scope (full halt or per-component), and copy the resulting preimage hash and bytes for submission on the **Whitelisted Caller** track. No local toolchain required, useful when the on-call operator is away from their dev machine.
* **`snowbridge-preimage` CLI**: local Rust binary in the [snowbridge](https://github.com/Snowfork/snowbridge) repo under `control/preimage`. Same halt scopes as the UI, useful for scripted runs or when the governance page is unreachable. Setup and full flag reference are in the [Emergency Pause](#emergency-pause) section below.

Both paths produce the same on-chain outcome: a preimage to submit on the **Whitelisted Caller** track. The flag semantics described under [Emergency Pause](#emergency-pause) apply to the UI form fields as well.

## Emergency Pause

In case of an emergency, a call to halt the bridge needs to be executed as soon as possible. Deploying the fix and resuming the bridge can happen afterwards with less time pressure and sensitivity, similar to a normal upgrade.

The CLI process is:

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

The command will produce a preimage hash, to be submitted to the **Whitelisted Caller Track**:

```
Preimage Hash: 0xc2569b432fba3b01df7da3c90bb546158480067064d4d5bc88c351fcba4355dd
Preimage Size: 129
0x1a0408630003000100a90f03242f00000602b28d0b9859460c530101200006028217b42..
```

The halt-bridge command has the following flags:

* `--all` Halt all the bridge components at once. This is the default when no other flag is given.
* `--ethereum-client` Halt the Ethereum beacon light client. Blocks new beacon-header ingestion via `EthereumBeaconClient::submit` **and** short-circuits `Verifier::verify` for every downstream consumer on BridgeHub. Concretely this stops:
  * inbound V1 `submit` and inbound V2 `submit` (Ethereum → Polkadot messages), and
  * `outbound-queue-v2::submit_delivery_receipt` (relayer-reward payouts against `PendingOrders`).

  This is the single lever to pull during a suspected beacon-light-client or sync-committee compromise. `force_checkpoint` remains available (root-only) for recovery.
* `--inbound-queue` Halt both V1 and V2 inbound-queue pallets on BridgeHub, blocking processing of Ethereum → Polkadot messages. For surgical halts of a single version, use the two flags below.
* `--inbound-queue-v1` Halt only the V1 inbound-queue pallet on BridgeHub.
* `--inbound-queue-v2` Halt only the V2 inbound-queue pallet on BridgeHub.
* `--outbound-queue` Halt AssetHub → Ethereum outbound traffic. Halts the V1 outbound-queue pallet on BridgeHub **and** the system-frontend pallet on AssetHub; the latter short-circuits the AssetHub → Ethereum `PausableExporter` for both V1 and V2 at the XcmRouter layer. (V2's `outbound-queue-v2` has no local halt, so the system-frontend halt is the primary V2 outbound lever.)
* `--system-frontend` Router-layer P→E halt: halts only the AssetHub system-frontend pallet. Blocks **both** V1 and V2 P→E at the `PausableExporter` (returns `SendError::NotApplicable` to the XcmRouter). The V1 BridgeHub outbound-queue is left untouched, so in-flight V1 messages already enqueued there keep draining. There is no V2-only operating-mode P→E halt; use `--assethub-max-fee-v2` for a V2-only deterrent.
* `--gateway` Halt the Ethereum Gateway contract (both V1 and V2 paths). Sends `Command::SetOperatingMode(Halted)` via both V1 and V2 system pallets so the halt is delivered via whichever outbound queue is live. Delivery is relayer-dependent, so schedule this **before** any local outbound halt takes effect.
* `--gateway-v2` V2-only Gateway halt: sends `Command::SetOperatingMode(Halted)` only via the V2 system pallet. Once delivered to Ethereum, blocks `v2_sendMessage` and `v2_registerToken`; leaves V1 `sendToken`/`sendMessage` working. Pair with `--inbound-queue-v2` and `--assethub-max-fee-v2` for the closest thing to a V2-only pause (no V2-only operating-mode P→E halt exists).
* `--assethub-max-fee` Set the AssetHub → Ethereum outbound fee to `u128::MAX` for both V1 (`BridgeHubEthereumBaseFee`) and V2 (`BridgeHubEthereumBaseFeeV2`), effectively deterring user sends via fee pricing. Complementary to the system-frontend halt; does not block at the router layer.
* `--assethub-max-fee-v2` V2-only variant of `--assethub-max-fee`: writes only `BridgeHubEthereumBaseFeeV2`, leaving V1 fee untouched. This is the only V2-isolated P→E lever (fee deterrent, not a hard halt).

Based on the nature of the emergency, the bridge might need to be halted in its entirety, or partially. Pick the narrowest lever that covers the suspected failure mode:

| Suspected failure mode | Minimum-scope command |
| --- | --- |
| Beacon light client / sync committee compromise | `halt-bridge --ethereum-client` |
| Ethereum Gateway contract compromise | `halt-bridge --gateway --assethub-max-fee` |
| Inbound-queue bug (one version) | `halt-bridge --inbound-queue-v1` or `--inbound-queue-v2` |
| Outbound-queue / system-frontend bug | `halt-bridge --outbound-queue` |
| V2 P→E only (V1 keeps flowing) | `halt-bridge --assethub-max-fee-v2` (fee deterrent, no operating-mode halt) |
| Full V2 pause (V1 keeps flowing, P→E is fee-deterrent only) | `halt-bridge --gateway-v2 --inbound-queue-v2 --assethub-max-fee-v2` |
| Full P→E halt (both V1 and V2) | `halt-bridge --outbound-queue` or `halt-bridge --system-frontend` |
| Uncertain / any component suspect | `halt-bridge --all` |

In case of emergency where there is uncertainty of the cause of a problem, it is best to block the bridge in its entirety using `halt-bridge --all`. To block both transfer directions at the earliest point possible, use `halt-bridge --gateway --assethub-max-fee`.

## Emergency Upgrade

Although unlikely, there may be scenarios where the emergency can only be resolved through an upgrade rather than just a pause. In this case, the code for the upgrade may be sensitive, and we would want to avoid overly publicising it until the fix has been executed.
