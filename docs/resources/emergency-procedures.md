# Emergency Procedures

This page is the on-call runbook for Snowbridge emergency response. It covers detection, decision-making, comms, the two emergency scenarios (pausing the bridge and emergency upgrades), and post-incident steps. For routine governance and the broader governance model, see [Governance and Operational Processes](governance-and-operational-processes.md).

The Snowbridge team is small (three people) and operates all-hands during an incident. Halting the bridge is technically reversible (it's a non-destructive operating-mode flip), but the halt referendum appearing on Polkassembly is itself a public signal that something is wrong, so it isn't free. The thresholds in [Decision authority](#decision-authority) are calibrated accordingly: solo action is reserved for cases where the incident is already publicly visible (funds being drained, active exploit), and confirmation-before-action is the default for everything else.

## Detection

Anything on this list is a valid trigger to start the incident flow. When in doubt, post in Slack and treat it as an incident until ruled out.

* **Funds drained or unexpectedly moved** on either side of the bridge (highest priority, halt first, investigate after).
* **Bug bounty report** (Immunefi or direct) describing an exploitable vulnerability, especially one with a proof-of-concept.
* **Unexpected Gateway events on Ethereum**: messages with unknown nonces, forged commands, or `Command::*` events not produced by a known relayer run.
* **Beacon light-client anomalies**: stalled or non-progressing beacon headers on BridgeHub, unexpected sync-committee rotation, or failures in `EthereumBeaconClient::submit`.
* **Queue depth or message-processing anomalies**: inbound/outbound queues backing up, repeated `Verifier::verify` failures, or messages stuck in pending state.
* **User reports** of failed or missing transfers that don't match a known issue, especially from integration partners (Hydration, others).
* **Anything one of the three of you doesn't immediately understand** and can't rule out within a few minutes. Halt first.

## Decision authority

To keep the path to halt short without creating unnecessary alarm:

* **Solo halt** (any 1 of 3, no confirmation needed): reserved for cases where the incident is already publicly visible and acting fast matters more than coordinating. Concretely: **funds are visibly being drained on-chain, or an active exploit is in progress**. In these cases the halt referendum appearing on-chain isn't new information to the community, the exploit already is.
* **Confirmed halt** (at least 2 of 3 agree before submitting): the default for everything else, including bug bounty reports without active exploitation, anomalies, unexplained queue behaviour, and "I don't understand what I'm seeing." A halt referendum appearing on Polkassembly is itself a public signal that something is wrong, so absent a visible exploit it's worth a few minutes of Slack discussion to confirm before submitting.
* **Escalating to Parity**: at least 2 of 3 team members agree there is a genuine incident. Same threshold as a confirmed halt; in practice these decisions happen together. This is the gate for creating the cross-team Element channel.
* **Public comms**: all three team members agree, and only after the fix is deployed and the bridge is resuming. See [Comms during an incident](#comms-during-an-incident).
* **Emergency upgrade**: coordinated decision with Parity once they are in the loop, since upgrade code is often exploit-sensitive.

## Comms during an incident

Order matters. Each step assumes the previous one has happened.

1. **Team Slack**: post what you've seen in the team channel, include links to the signal (block explorer, alert, bounty report). For non-visible signals (bug reports, anomalies), wait for at least one teammate to confirm before submitting a halt referendum; see [Decision authority](#decision-authority). For a visible exploit or funds being drained, skip ahead and halt now.
2. **Halt the bridge** (if warranted) using the steps in [Producing a halt-bridge preimage](#producing-a-halt-bridge-preimage) and [Submitting the preimage](#submitting-the-preimage). For visible exploits, do this in parallel with steps 3 and 4 below.
3. **Internal confirmation**: once 2 of 3 team members have agreed there is a genuine incident, proceed to step 4. For solo-halt cases this is retroactive: the other two confirm as they come online.
4. **Element channel with Parity**: create a new Element room and invite the Parity Bridges contacts (Adrian, Bastian, Oliver). This is where Fellowship coordination for the Whitelisted Caller submission happens, and where Parity gets the context they need to help.
5. **Affected integrators**: contact integration partners directly (Hydration first, then others) once Parity is in the loop. Channel: whatever direct line you have with them (Element, Slack Connect, Signal). Tell them what's halted and the expected resume timing.
6. **No public comms** (Twitter/X, forum, blog, public Discord) until the fix is deployed and resume is in flight. Premature public disclosure of an unfixed vulnerability is worse than silence.

## Producing a halt-bridge preimage

**Retrieve the halt-bridge preimage from the governance page at [app.snowbridge.network/governance](https://app.snowbridge.network/governance).** Select the halt scope (full halt or per-component), and copy the resulting preimage hash and bytes for submission on the **Whitelisted Caller** track. No local toolchain required, which matters when the on-call operator is away from their dev machine.

The flag semantics described under [Emergency Pause](#emergency-pause) apply to the governance page's form fields as well; that section is the canonical reference for what each scope does.

**Fallback only if the governance page is unreachable**: the `snowbridge-preimage` CLI in the [snowbridge](https://github.com/Snowfork/snowbridge) repo under `control/preimage` produces the same preimage bytes. Setup and full flag reference are in the [Emergency Pause](#emergency-pause) section below. Use it only when the UI is down, since the UI is the single source of truth the team is expected to drive from during an incident.

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

## Submitting the preimage

Halt preimages go through Polkadot OpenGov's **Whitelisted Caller** track. This is the fastest emergency track available, but it is not instantaneous:

1. **Submit the preimage** on Polkadot via Polkassembly/Subsquare or directly via the preimage pallet, using the bytes from the previous step. Note the preimage hash.
2. **Whitelist the call hash**: the Whitelisted Caller track requires the specific call hash to be whitelisted by the Polkadot Fellowship via a Fellowship referendum. This is the bottleneck. In the Element channel with Parity (Adrian, Bastian, Oliver), share the preimage hash and ask them to coordinate a Fellowship referendum to whitelist it. Parity has Fellowship members and direct lines to others.
3. **Submit the Whitelisted Caller referendum** referencing the preimage. Once the Fellowship referendum to whitelist passes, the Whitelisted Caller referendum's confirmation period begins.
4. **Wall-clock expectation**: from preimage produced to call executed on-chain depends on Fellowship response time. Plan for the order of hours, not minutes, even on the emergency track. This is why the order in [Comms during an incident](#comms-during-an-incident) puts Parity escalation in parallel with halt submission.

## Verifying the halt

Once the call executes, verify the halt actually took effect on each affected chain. Don't trust the referendum closing as sufficient evidence, query state.

Per-component checks, all reading the pallet's `OperatingMode` storage item (expected value: `Halted`):

| Halt flag | Chain | Storage item to query |
| --- | --- | --- |
| `--ethereum-client` | BridgeHub | `ethereumBeaconClient.operatingMode` |
| `--inbound-queue-v1` | BridgeHub | `ethereumInboundQueue.operatingMode` |
| `--inbound-queue-v2` | BridgeHub | `ethereumInboundQueueV2.operatingMode` |
| `--outbound-queue` (BridgeHub side) | BridgeHub | `ethereumOutboundQueue.operatingMode` |
| `--outbound-queue` / `--system-frontend` (AssetHub side) | AssetHub | `systemFrontend.operatingMode` (the snowbridge-system-frontend pallet) |
| `--gateway` (local pallet state) | BridgeHub | `ethereumSystem.operatingMode` and `ethereumSystemV2.operatingMode` |
| `--gateway` (actual contract state on Ethereum) | Ethereum | `Gateway.operatingMode()` returns `Halted`. **This step depends on relayer delivery**, so the on-chain Polkadot referendum executing does not mean the Ethereum contract is halted yet. Watch for the `SetOperatingMode` event on the Gateway and confirm operating mode after expected delivery. |
| `--assethub-max-fee` | AssetHub | `bridgeHubEthereumBaseFee` and `bridgeHubEthereumBaseFeeV2` equal `u128::MAX` (`340282366920938463463374607431768211455`) |
| `--assethub-max-fee-v2` | AssetHub | `bridgeHubEthereumBaseFeeV2` equals `u128::MAX` |

If the Polkadot side is halted but the Gateway contract is not (because no relayer has delivered the `SetOperatingMode` command yet), users can still call `sendToken` / `sendMessage` on Ethereum, those messages just won't be processed downstream. Treat the Polkadot-side halt as the firm guarantee; the Gateway-side halt as a follow-up.

## Resuming the bridge

Once the fix is deployed and the root cause is understood, **retrieve the resume-bridge preimage from the same governance page at [app.snowbridge.network/governance](https://app.snowbridge.network/governance)**. Select the resume action and the scopes to resume (typically matching whatever was halted), and copy the preimage hash and bytes for submission on the **Whitelisted Caller** track.

The `snowbridge-preimage` CLI does not currently have a top-level `resume-bridge` subcommand, so the governance page is the only practical path. If you need to script it (rare), the underlying SDK function is `buildResumeBridgePreimage` in `@snowbridge/api`.

Before submitting the resume preimage, confirm:

* The fix is deployed and verified in production (runtime upgrade live, contracts upgraded if applicable).
* All three team members have signed off on resuming.
* Monitoring is back to normal baselines and there are no fresh anomalies.
* AssetHub fee values are being restored to their pre-incident values (the resume preimage writes `BridgeHubEthereumBaseFee` / `BridgeHubEthereumBaseFeeV2` back to known good defaults, currently `14_929_540_998` for V1 and `1_000_000_000` for V2; double-check these match what was live before the incident).

The resume preimage submission follows the same Whitelisted Caller flow as [Submitting the preimage](#submitting-the-preimage). Public comms can begin once the resume call has executed and the bridge is observed processing messages again.

## Emergency Upgrade

Although unlikely, there may be scenarios where the emergency can only be resolved through an upgrade rather than a pause, for example a critical bug in a pallet's logic that even a halt doesn't fully contain. In this case:

* **Halt first anyway**. A halt buys time to develop, review, and deploy the upgrade without time pressure. The exceptions are situations where the halt itself would be more harmful than the bug, which is rare.
* **Keep the upgrade code restricted**. Upgrade code for a known unpatched vulnerability is itself exploit material. Develop in a private branch, limit reviewers to the team plus the necessary Parity contacts, and avoid publicising the patch until it has executed on-chain.
* **Coordinate with Parity** on the upgrade pathway (Whitelisted Caller for runtime upgrades, multi-sig for contract upgrades, etc.). The Element channel from [Comms during an incident](#comms-during-an-incident) is the right venue.
* **Resume the bridge** only after the upgrade is verified live, following the [Resuming the bridge](#resuming-the-bridge) checklist.

## Post-mortem

Within 48 hours of the bridge being resumed (or sooner if everyone has bandwidth):

* **Owner**: whichever team member drove the incident (whoever halted first by default).
* **Format**: Google Doc, shared with the team and the Parity contacts who were in the Element channel.
* **Minimum contents**: timeline of events (with timestamps), root cause, what the halt scope was and why, what worked in the response, what didn't, action items with owners and target dates.
* **Action items**: track in the team's normal issue tracker, not the doc itself, so they don't get lost when the doc goes stale.

Even for false-positive halts (incident turned out to be non-incident), write a short post-mortem. Tuning detection signals to reduce false positives is itself a useful output.
