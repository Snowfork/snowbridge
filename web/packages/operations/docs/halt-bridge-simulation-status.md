# Halt + Resume Bridge SDK Simulation

Two-phase simulation that exercises the halt-bridge and resume-bridge SDK preimages end-to-end against a forked Polkadot AssetHub + BridgeHub (chopsticks) and Ethereum mainnet (Foundry/anvil). Used to demo or sanity-check the preimages before submitting them to OpenGov.

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

In addition, two negative tests assert the halt actually blocks user-facing sends, not just that the bit is flipped:

- **AH frontend rejects P→E**: Phase 1 submits a signed `snowbridgeSystemFrontend.registerToken` extrinsic from Alice (chopsticks accepts mock signatures) and asserts the dispatch errors with `snowbridgeSystemFrontend.Halted` ("Message export is halted").
- **Gateway rejects V2 send**: Phase 2 calls `v2_sendMessage` from a regular EOA and asserts the call reverts with the `Disabled()` selector (`0x75884cda`). The V1 `sendToken` assertion is best-effort: it probes a list of well-known mainnet ERC20s via `quoteSendTokenFee` to find one the Gateway has registered. If none of them are registered at the fork block, the V1 assertion is skipped, since unregistered tokens would revert with `TokenNotRegistered` and not actually test the mode gate. The V1 and V2 send paths share the same global `CoreStorage.layout().mode`, so V2's `Disabled()` revert is sufficient evidence that the halt gates both.

After the halt-side checks, Phase 1 builds a **resume preimage** via `governance.buildResumeBridgePreimage(ah, bh, {all: true})`, dispatches it through the same scheduler-injection mechanism, and asserts the inverse of every halt assertion:

- Every BH `operatingMode` storage item is back to `Normal`.
- AH `snowbridgeSystemFrontend.exportOperatingMode` is back to `Normal`.
- `:BridgeHubEthereumBaseFee:` / `:BridgeHubEthereumBaseFeeV2:` storage values match the **exact bytes** that were on-chain before the halt (captured pre-halt in the same run). This catches drift between the SDK's hardcoded `PROD_BASE_FEE_V1` / `PROD_BASE_FEE_V2` constants and what prod actually holds.
- The same `snowbridgeSystemFrontend.registerToken` call that previously failed with `Halted` now fails for an unrelated downstream reason (`BadOrigin` with throwaway args), proving the halt gate is no longer firing.

Phase 2 then replays the V2 resume message captured by Phase 1: `v2_dispatchCommand` with `payload = abi.encode(uint8(0))`. The deployed Gateway accepts the V2 unhalt (flips `operatingMode` to `Normal`), and a subsequent `v2_sendMessage` eth_call no longer reverts with `Disabled()`.

Phase 2 intentionally does **not** use the V1 path to reset operating mode back to `Normal`: on the deployed mainnet Gateway, sending a V1 `setOperatingMode(Normal)` via `eth_sendTransaction` from the impersonated Gateway address reverts (even though `eth_call` simulates it as a success). V2 dispatch works for both halt and unhalt, so the resume Phase 2 test uses V2.

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
