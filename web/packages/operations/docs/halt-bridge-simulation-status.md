# Halt-Bridge SDK Simulation

Two-phase simulation that exercises the halt-bridge SDK end-to-end against a forked Polkadot AssetHub + BridgeHub (chopsticks) and Ethereum mainnet (Foundry/anvil). Used to demo or sanity-check a halt preimage before submitting it to OpenGov.

## What it tests

For the SDK's `{all: true}` halt lever, all 9 halt items are verified at the storage level (events alone are not sufficient: an `OperatingModeChanged` event would also fire if mode were set back to `Normal`, and `system.setStorage` emits no event).

| # | Halt item | How it's verified |
|---|---|---|
| 1 | Gateway V1 (BH `ethereumSystem.setOperatingMode`) | BH `MessageQueued` event + Phase 2 reads `Gateway.operatingMode == RejectingOutboundMessages` after replaying the queued bytes |
| 2 | Gateway V2 (BH `ethereumSystemV2.setOperatingMode`) | same as #1, via V2 path |
| 3 | BH `ethereumInboundQueue.operatingMode == Halted` | event + Phase 1 storage read |
| 4 | BH `ethereumInboundQueueV2.operatingMode == Halted` | event + Phase 1 storage read |
| 5 | BH `ethereumOutboundQueue.operatingMode == Halted` | event + Phase 1 storage read |
| 6 | BH `ethereumBeaconClient.operatingMode == Halted` | event + Phase 1 storage read |
| 7 | AH `snowbridgeSystemFrontend.exportOperatingMode == Halted` | event + Phase 1 storage read |
| 8 | AH `:BridgeHubEthereumBaseFee:` = u128::MAX | Phase 1 reads `state.getStorage(twox_128(":BridgeHubEthereumBaseFee:"))` |
| 9 | AH `:BridgeHubEthereumBaseFeeV2:` = u128::MAX | same, V2 key |

## What it does not test

Two layers are stubbed:

1. **Governance landing.** Phase 1 uses chopsticks' `dev_setStorage` to inject the preimage as `Noted` and the agenda entry as `Root`, then builds the dispatch block manually. This proves the call itself works; it does not exercise the Whitelisted Caller track's origin filter.
2. **BEEFY commitment + MMR proof verification.** Phase 2 uses `anvil_impersonateAccount` to call the Gateway's inner handlers as Gateway-from-Gateway (the `onlySelf` modifier permits this) rather than going through `submitV1` / `submitV2`. This proves the post-proof dispatch path; it does not exercise the BEEFY verifier. Producing a valid BEEFY commitment on a fork is not feasible without a relay-chain validator set.

## Phase 1, Polkadot side

`web/packages/operations/src/halt_bridge_simulation.ts`

1. Connect to forked AssetHub (`ws://localhost:8000`) and BridgeHub (`ws://localhost:8001`).
2. Build the preimage via `governance.buildHaltBridgePreimage(ah, bh, {all: true})`.
3. Verify `blake2_256(callData) === preimage.hash`.
4. Inject the preimage as `Noted` and schedule the call with `Root` for the next block, via `dev_setStorage`.
5. Build AH blocks, wait for HRMP relay to BH, build BH blocks.
6. Assert the expected events on both chains (`scheduler.Dispatched`, `polkadotXcm.Sent`, the eight BH `*OperatingMode*` / `MessageQueued` events).
7. Extract the V1 + V2 queued messages from BH storage and write them to a handoff JSON for Phase 2.
8. Assert the storage values listed in the table above.

Run: `pnpm haltBridgeSimulation`. Env overrides: `ASSET_HUB_WS`, `BRIDGE_HUB_WS`, `HALT_BRIDGE_HANDOFF`.

## Phase 2, Ethereum side

`web/packages/operations/src/halt_bridge_simulation_ethereum.ts`

1. Connect to anvil (`http://localhost:8545`) forking Ethereum mainnet.
2. Load the Phase 1 handoff JSON (or fall back to hardcoded `abi.encode(uint8(1))` and `ZeroHash` if not present).
3. Impersonate the Gateway address via `anvil_impersonateAccount`.
4. V1 path: call `v1_handleSetOperatingMode(handoff.v1.params)`, assert `gateway.operatingMode() == RejectingOutboundMessages`.
5. Reset mode to `Normal`.
6. V2 path: call `v2_dispatchCommand((kind, gas, payload), origin)` with the handoff values, assert mode flipped again.

Run: `pnpm haltBridgeSimulationEthereum`. Env overrides: `ETHEREUM_RPC`, `GATEWAY_ADDRESS`, `HALT_BRIDGE_HANDOFF`.

## Phase 1 → Phase 2 handoff

Phase 1 writes `/tmp/halt-bridge-sim/messages.json` (override with `HALT_BRIDGE_HANDOFF`) containing the V1 `params` bytes and the V2 `(kind, gas, payload, origin)` tuple read from `ethereumOutboundQueue::Messages` / `ethereumOutboundQueueV2::Messages` at the BH block hash where `MessageQueued` fired. Phase 2 replays those exact bytes against the Gateway, so the calldata going through the handlers is byte-for-byte what would arrive in production after relaying.

The storage has to be read at the BH block hash where the message was queued (Phase 1 captures it from the event subscription), not at the latest head — those storage items are cleared the block after the message is queued.

## Running it

### 1. Chopsticks (AH + BH, XCM mode)

Configs live in `chopsticks/`. AH is exposed on `:8000`, BH on `:8001`.

```
cd web/packages/operations
npx @acala-network/chopsticks@latest xcm -p chopsticks/asset-hub.yml -p chopsticks/bridge-hub.yml
```

First start is slow (1-2 min) while chopsticks fetches the runtime + state from the RPCs.

### 2. Anvil (Ethereum mainnet fork)

```
LATEST=$(curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' https://ethereum-rpc.publicnode.com | grep -oE '0x[0-9a-f]+' | head -1)
PIN=$(( LATEST - 10 ))
anvil --fork-url https://ethereum-rpc.publicnode.com --fork-block-number $PIN
```

Pinning ~10 blocks behind tip avoids `historical state ... is not available` errors once the public RPC prunes the fork point. With an archive RPC, the pin is unnecessary.

### 3. Run

```
cd web/packages/operations
pnpm haltBridgeSimulation
pnpm haltBridgeSimulationEthereum
```
