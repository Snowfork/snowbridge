# Emergency Procedures

On-call runbook. Halt mechanics are at the top so they're easy to reach under pressure. For routine governance, see [Governance and Operational Processes](governance-and-operational-processes.md).

Halt is technically reversible but the halt referendum on Polkassembly is public. See [Decision authority](#decision-authority) for when solo action vs team confirmation applies.

## Producing a halt-bridge preimage

Get the preimage from [app.snowbridge.network/governance](https://app.snowbridge.network/governance). Select the halt scope (see [Halt scopes reference](#halt-scopes-reference)), copy the hash and bytes, and feed bytes into `opengov-cli` (see [Submitting the preimage](#submitting-the-preimage)).

**Fallback (UI down only)**: call `buildHaltBridgePreimage` from `@snowbridge/api` with a `HaltBridgeOptions` matching the same scopes.

## Halt scopes reference

Pick the narrowest scope that covers the failure mode. Governance page form fields:

* **All** Every component. Default if nothing else selected.
* **Ethereum client** Halts `EthereumBeaconClient::submit` and short-circuits `Verifier::verify` for all BridgeHub consumers. Stops V1 + V2 inbound `submit` and `outbound-queue-v2::submit_delivery_receipt`. Use for beacon-light-client or sync-committee compromise. `force_checkpoint` stays available (root-only) for recovery.
* **Inbound queue** Both V1 + V2 inbound pallets on BridgeHub.
* **Inbound queue V1** V1 inbound only.
* **Inbound queue V2** V2 inbound only.
* **Outbound queue** V1 outbound on BridgeHub **and** AssetHub system-frontend (short-circuits `PausableExporter` for V1 + V2 at XcmRouter). V2 has no local outbound halt, so system-frontend is the primary V2 outbound lever.
* **System frontend** AssetHub system-frontend only. Blocks V1 + V2 P→E at `PausableExporter` (`SendError::NotApplicable`). V1 BridgeHub outbound keeps draining in-flight messages.
* **Gateway** Sends `Command::SetOperatingMode(Halted)` to the Ethereum Gateway via both V1 + V2 system pallets. Delivery is relayer-dependent, so schedule **before** local outbound halts.
* **Gateway V2** V2-only Gateway halt. Blocks `v2_sendMessage` and `v2_registerToken` once delivered. Pair with **Inbound queue V2** + **AssetHub max fee V2** for a V2-only pause.
* **AssetHub max fee** Sets `BridgeHubEthereumBaseFee` + `BridgeHubEthereumBaseFeeV2` to `u128::MAX`. Fee deterrent, not a router halt.
* **AssetHub max fee V2** V2-only variant. Writes only `BridgeHubEthereumBaseFeeV2`. Only V2-isolated P→E lever.

| Failure mode | Scopes |
| --- | --- |
| Beacon light client / sync committee compromise | **Ethereum client** |
| Ethereum Gateway compromise | **Gateway** + **AssetHub max fee** |
| Inbound-queue bug (one version) | **Inbound queue V1** or **Inbound queue V2** |
| Outbound-queue / system-frontend bug | **Outbound queue** |
| V2 P→E only (V1 keeps flowing) | **AssetHub max fee V2** (fee deterrent only) |
| Full V2 pause | **Gateway V2** + **Inbound queue V2** + **AssetHub max fee V2** |
| Full P→E halt (V1 + V2) | **Outbound queue** or **System frontend** |
| Uncertain | **All** |

When uncertain: **All**. To block both directions immediately: **Gateway** + **AssetHub max fee**.

## Submitting the preimage

Goes through OpenGov **Whitelisted Caller** track. Requires Polkadot Fellowship to whitelist the call first. Use [opengov-cli](https://github.com/joepetrowski/opengov-cli):

{% code overflow="wrap" %}
```
opengov-cli submit-referendum \
    --proposal 0x<preimage-bytes> \
    --network polkadot \
    --track whitelistedcaller \
    --output AppsUiLink
```
{% endcode %}

Outputs two pre-built batches as papi.how links:

1. **Polkadot Asset Hub batch** Preimage note + public Whitelisted Caller referendum. Anyone on the team can submit.
2. **Polkadot Collectives Chain batch** Opens Fellowship whitelist referendum. **Requires rank-3+ Fellow.** Coordinate with Parity (Adrian, Bastian, Oliver) in Element. Bottleneck of the flow.

Enactment defaults to `After(10)` blocks. Override with `--at <block>` or `--after <blocks>`.

**Wall-clock: hours, not minutes.** Run Parity escalation in parallel with submission (see [Comms](#comms-during-an-incident)).

## Verifying the halt

After the call executes, query each affected chain's `OperatingMode` storage (expected: `Halted`).

| Halt scope | Chain | Storage |
| --- | --- | --- |
| **Ethereum client** | BridgeHub | `ethereumBeaconClient.operatingMode` |
| **Inbound queue V1** | BridgeHub | `ethereumInboundQueue.operatingMode` |
| **Inbound queue V2** | BridgeHub | `ethereumInboundQueueV2.operatingMode` |
| **Outbound queue** (BridgeHub) | BridgeHub | `ethereumOutboundQueue.operatingMode` |
| **Outbound queue** / **System frontend** (AssetHub) | AssetHub | `systemFrontend.operatingMode` |
| **Gateway** (BridgeHub side) | BridgeHub | `ethereumSystem.operatingMode`, `ethereumSystemV2.operatingMode` |
| **Gateway** (Ethereum contract) | Ethereum | `Gateway.operatingMode() == Halted`. **Relayer-dependent**: watch for `SetOperatingMode` event before confirming. |
| **AssetHub max fee** | AssetHub | `bridgeHubEthereumBaseFee` + `bridgeHubEthereumBaseFeeV2` == `u128::MAX` (`340282366920938463463374607431768211455`) |
| **AssetHub max fee V2** | AssetHub | `bridgeHubEthereumBaseFeeV2` == `u128::MAX` |

Polkadot-side halt is the firm guarantee. If Gateway isn't halted yet (no relayer delivery), `sendToken`/`sendMessage` on Ethereum still accept calls but nothing downstream processes them.

## Detection

Triggers for the incident flow:

* **Funds drained or unexpectedly moved**. Highest priority. Halt first, investigate after.
* **Bug bounty report** (HackenProof or direct), verified by a team member as a valid exploit with working PoC.

When in doubt: post in Slack, treat as incident until ruled out.

## Decision authority

| Action | Threshold |
| --- | --- |
| Solo halt | 1 member. **Only for visible exploit / funds being drained.** |
| Confirmed halt | 2 members agree. Default for bug bounty, anomalies, "I don't understand what I'm seeing." |
| Escalate to Parity | 2 members agree. Same conversation as confirmed halt in practice. |
| Public comms | Full team. **Only after fix is deployed and bridge is resuming.** |
| Emergency upgrade | Coordinated with Parity. Code is exploit-sensitive. |

A halt referendum on Polkassembly is public, so solo authority is reserved for cases where the incident is already public (funds moving). Otherwise discuss in Slack first.

## Comms during an incident

Each step assumes the previous one has happened.

1. **Slack** `#snowbridge-security` Post the signal (link to explorer, alert, bounty report). Non-visible signals: wait for at least one teammate to confirm before halting. Visible exploit: skip ahead.
2. **Halt** See [Producing](#producing-a-halt-bridge-preimage) + [Submitting](#submitting-the-preimage). For visible exploits, run in parallel with steps 3 and 4.
3. **Internal confirmation** 2+ members agree it's a real incident. Retroactive for solo-halt cases.
4. **Element with Parity** New room, invite Adrian, Bastian, Oliver. Fellowship coordination happens here.
5. **Integrators** Telegram. Hydration first, then others. Tell them what's halted + expected resume timing.
6. **No public comms** (Twitter/X, forum, blog, public Discord) until fix is deployed and resume is in flight.

## Resuming the bridge

Get the resume preimage from [app.snowbridge.network/governance](https://app.snowbridge.network/governance). Select scopes matching what was halted. Submission: same `opengov-cli` + Fellowship Whitelist flow as [Submitting the preimage](#submitting-the-preimage).

Fallback: `buildResumeBridgePreimage` in `@snowbridge/api`.

Before submitting, confirm:

* Fix is deployed and verified in production.
* Full team has signed off.
* Monitoring is back to baseline, no fresh anomalies.
* AssetHub fee values being restored to pre-incident values. Resume writes `BridgeHubEthereumBaseFee` and `BridgeHubEthereumBaseFeeV2` back to known good defaults (currently `14_929_540_998` for V1, `1_000_000_000` for V2). Double-check these match what was live before.

Public comms can begin once resume executes and the bridge is processing again.

## Emergency Upgrade

For cases a halt alone can't contain (e.g. critical pallet logic bug):

* **Halt first anyway.** Buys time to develop the upgrade without pressure. Skip only if halting is itself harmful.
* **Restrict the code.** Upgrade code for an unpatched vulnerability is itself exploit material. Private branch, limited reviewers (team + necessary Parity contacts). Don't publicise until executed on-chain.
* **Coordinate the pathway with Parity.** Whitelisted Caller for runtime, multi-sig for contracts. Use the Element channel.
* **Resume** only after the upgrade is verified live ([Resuming the bridge](#resuming-the-bridge)).

## Post-mortem

Within 48h of resume:

* **Owner** Whoever drove the incident (defaults to whoever halted first).
* **Format** Google Doc, shared with team + Parity contacts from the Element channel.
* **Contents** Timeline (timestamps), root cause, halt scope + reason, what worked, what didn't, action items with owners and dates.
* **Action items** Track in the team issue tracker, not the doc itself.

Also write one for false-positive halts. Tuning detection signals to reduce false positives is itself useful output.
