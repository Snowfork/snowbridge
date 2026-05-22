---
name: snowbridge-referendum
description: Generate a Snowbridge preimage, create opengov referendum calls, test with referenda-tester (including XCM verification on BridgeHub), and report results
disable-model-invocation: true
allowed-tools: Bash, Read, Write, Grep, Glob
argument-hint: <snowbridge-subcommand> [--track <track>] [--network <network>]
---

# Snowbridge Referendum Workflow

End-to-end workflow: generate a Snowbridge governance preimage, create opengov referendum calls with opengov-cli, test them with the referenda-tester (including XCM relay verification on BridgeHub), and report results ready for on-chain submission.

## Arguments

- `$0`: The snowbridge-preimage subcommand (e.g. `mint-feb2026`, `upgrade-v2`, `pricing-parameters`, etc.)
- Remaining args (`$ARGUMENTS`): May include `--track` and `--network` overrides.
  - `--track`: `whitelisted-caller` (default) or `root`
  - `--network`: `polkadot` (default), `kusama`, `westend`, `paseo`

## Project Locations

- Snowbridge preimage tool: `./preimage/` (relative to this control directory)
- opengov-cli: `../../opengov-cli/` (relative to this control directory)

## Steps

### 1. Generate the preimage

Run the snowbridge preimage tool:

```bash
cd <snowbridge-control-dir>/preimage && cargo run --features <network> -- <subcommand> [extra-flags]
```

- The `--features` flag must match the network: `polkadot`, `kusama`, `westend`, or `paseo`.
- Capture the hex output (the line starting with `0x`) from stdout.
- Capture the preimage hash and size from stderr (lines like `Preimage hash: 0x...` and `Preimage size: N`).
- Save the hex to `preimage.txt` in the opengov-cli project root.

### 2. Run opengov-cli

From the opengov-cli directory:

```bash
cargo run -- submit-referendum --proposal preimage.txt --network <network> --track <track> --output calldata
```

Parse the output to extract these hex values:
- **Fellowship referendum call**: line after "Open a Fellowship referendum to whitelist the call:" (only for whitelisted-caller track)
- **Governance batch call**: line after "Batch to submit on Polkadot Asset Hub:" (or the relevant chain)
- **Collectives batch call**: line after "Batch to submit on Polkadot Collectives Chain:" (only for whitelisted-caller track)

For `--track root`, there is no fellowship call or collectives batch.

### 3. Test with referenda-tester

This is the critical verification step. The referenda-tester simulates the full governance flow using Chopsticks.

#### Determine additional chains

If the preimage sends XCM to other chains (e.g. BridgeHub), add them via `--additional-chains`. This is essential for verifying the XCM roundtrip completes successfully.

Common patterns:
- **Preimage contains `PolkadotXcm::send` to BridgeHub** (e.g. mint, replay, upgrade commands): add BridgeHub
- **Preimage is AH-local only** (e.g. treasury spend, register asset): no additional chains needed

To determine this: check if the preimage tool's subcommand uses `send_xcm_bridge_hub()` in `commands.rs`. If so, BridgeHub is needed.

#### Whitelisted-caller track (default)

```bash
npx github:karolk91/polkadot-referenda-tester test \
  --governance-chain-url wss://asset-hub-polkadot-rpc.n.dwellir.com \
  --fellowship-chain-url wss://polkadot-collectives-rpc.polkadot.io \
  --call-to-create-governance-referendum '<governance-batch-hex>' \
  --call-to-create-fellowship-referendum '<collectives-batch-hex>' \
  --additional-chains wss://bridge-hub-polkadot-rpc.n.dwellir.com \
  --verbose
```

#### Root track

```bash
npx github:karolk91/polkadot-referenda-tester test \
  --governance-chain-url wss://asset-hub-polkadot-rpc.n.dwellir.com \
  --call-to-create-governance-referendum '<governance-batch-hex>' \
  --additional-chains wss://bridge-hub-polkadot-rpc.n.dwellir.com \
  --verbose
```

Omit `--fellowship-chain-url` and `--call-to-create-fellowship-referendum` for root track.
Omit `--additional-chains` if the preimage doesn't send XCM to other chains.

#### Testnet URLs

| Network | Asset Hub | Collectives | BridgeHub |
|---------|-----------|-------------|-----------|
| Polkadot | `wss://asset-hub-polkadot-rpc.n.dwellir.com` | `wss://polkadot-collectives-rpc.polkadot.io` | `wss://bridge-hub-polkadot-rpc.n.dwellir.com` |
| Kusama | `wss://asset-hub-kusama-rpc.n.dwellir.com` | `wss://kusama-collectives-rpc.n.dwellir.com` | `wss://bridge-hub-kusama-rpc.n.dwellir.com` |
| Westend | `wss://asset-hub-westend-rpc.n.dwellir.com` | `wss://westend-collectives-rpc.n.dwellir.com` | `wss://bridge-hub-westend-rpc.n.dwellir.com` |

### 4. Verify the referenda-tester output

Check the output for these success indicators:

#### Governance dispatch
- `Both referenda executed successfully!` (or single referendum for root track)
- `Scheduler.Dispatched` with `result: success`
- `Whitelist.WhitelistedCallDispatched` with `result: success` (whitelisted-caller track)

#### XCM to BridgeHub (if applicable)
- `PolkadotXcm.Sent` with destination `Parachain: 1002` (BridgeHub)
- `XcmpQueue.XcmpMessageSent` on Asset Hub
- Outbound HRMP message to recipient `1002` in the Chopsticks logs

#### BridgeHub processing (if additional chain added)
- `MessageQueue.Processed` with `origin: Sibling(1000)` and `success: true`
- `PolkadotXcm.Sent` from BridgeHub back to AH (destination `Parachain: 1000`)
- `XcmpQueue.XcmpMessageSent` on BridgeHub
- Outbound HRMP message to recipient `1000` in the Chopsticks logs

For commands that mint/deposit assets (e.g. `mint-feb2026`, `replay-sep2025`), verify the return XCM contains:
- `ReserveAssetDeposited` with the correct asset and amount
- `DepositAsset` with the correct beneficiary
- The asset location should be the expected foreign asset (e.g. Ethereum ERC20)

#### Failure investigation
If the test fails:
- Check `Scheduler.Dispatched` result for errors
- Look for `XcmpQueue.Fail` or `MessageQueue.ProcessingFailed` events
- Check BridgeHub events for `Transact` failures
- Inspect the Chopsticks `outboundHrmpMessage` logs to see if XCM was sent

### 5. Report results

Print a summary including:
- Preimage hash and size
- Whether all referenda passed
- Whether XCM roundtrip completed (if applicable)
- The final call data for posting:
  - Governance batch (submit on Asset Hub)
  - Collectives batch (submit on Collectives, whitelisted-caller only)
- Any issues found during testing

If everything passes, the calls are ready for on-chain submission.
