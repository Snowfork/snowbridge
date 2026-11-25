---
description: The exact runtime surface a parachain must expose for Snowbridge V2 token transfers.
---

# V2 Requirements

This is the complete list of what the Snowbridge V2 SDK (`@snowbridge/api`) calls on, sends to,
or expects from a non-Asset Hub parachain. It covers ERC-20s, Ether and Polkadot-native assets (PNAs) in both directions.
It is derived from the V2 transfer code (`toEthereumSnowbridgeV2.ts`, `toPolkadotSnowbridgeV2.ts`,
`transfers/toEthereum/*FromParachain.ts`, `transfers/toPolkadot/*ToParachain.ts`) and the registry
builder's feature detection. Each direction can be enabled on its own (see [Enabling](#enabling)).

Locations below are relative to the parachain unless stated otherwise.

| Name | Location |
|---|---|
| DOT | `{ parents: 1, interior: Here }` |
| Ether | `{ parents: 2, interior: X1(GlobalConsensus(Ethereum { chain_id })) }` |
| ERC-20 | `{ parents: 2, interior: X2(GlobalConsensus(Ethereum { chain_id }), AccountKey20 { network: None, key }) }` |
| Native token | `{ parents: 0, interior: Here }` |
| Asset Hub | `{ parents: 1, interior: X1(Parachain(1000)) }` |

## 1. Common to both directions

### Runtime APIs

| API | Method | Used for | Requirement |
|---|---|---|---|
| `XcmPaymentApi` | `query_xcm_weight` | Weighing every XCM the chain executes | Must weigh all XCM v5 instructions listed below. Must not return `Unimplemented` or a `MAX` weight. |
| `XcmPaymentApi` | `query_weight_to_asset_fee` | Converting weight to a fee | Must price **DOT** and **Ether**, plus the **native token** if it is offered as a fee asset. Queried with a `V4` asset ID, then retried with `V5`. |
| `DryRunApi` | `dry_run_call`, `dry_run_xcm` | Validation before signing | Strongly recommended. Without it, validation reports `DryRunNotSupportedOnDestination` and cannot confirm success. |
| `TransactionPaymentApi` | `query_info` | Source execution fee (`paymentInfo`) | Required for Polkadot → Ethereum. |
| RPC | `system_accountNextIndex`, `system_properties`, `system_chain` | Nonce and chain properties | Standard. |

### XCM configuration

- **XCM v5 executor.** The negotiated XCM version with Asset Hub must be 5 in both directions (`polkadotXcm.supportedVersion`).
- **`AliasOrigin` must be weighed below `MAX`.** The registry builder probes this with `query_xcm_weight([AliasOrigin(..)])` and sets `supportsAliasOrigin` from it.
- **Ether as a fee asset.** `query_weight_to_asset_fee(weight, Ether)` must succeed (the registry's `hasEthBalance`), and the trader must accept Ether in `PayFees` and `BuyExecution`.
- **Asset Hub is the trusted reserve for DOT, Ether and every ERC-20** (`IsReserve`), with those assets held by the chain's sovereign account on Asset Hub.
- **Open HRMP channels with Asset Hub in both directions.**

### Assets

- Ether and each bridged ERC-20 are registered in an asset pallet (`orml-tokens`, `pallet-assets` or similar) under **exactly** the locations above.
- An account that does not exist must be able to receive the bridged asset. Either the asset is sufficient, or the user must already hold the native token; otherwise validation reports `AccountDoesNotExist`.
- Existential deposits and minimum balances are read from the chain's own metadata.

## 2. Ethereum → parachain

The parachain only **receives** XCM from Asset Hub. It needs nothing else beyond Section 1.

### What arrives

Asset Hub executes the user's `InitiateTransfer { destination: parachain, preserve_origin: false }` and sends:

```
ReserveAssetDeposited([fee asset])   // Ether by default, or DOT if the user picks the DOT-fee option
PayFees(fee asset)
ReserveAssetDeposited([token])       // ERC-20 / Ether.  For a PNA native to this chain: ReceiveTeleportedAsset([token])
ClearOrigin
RefundSurplus
...custom XCM (optional)
DepositAsset(AllOf(token), beneficiary)
DepositAsset(AllCounted(1), beneficiary)   // leftover fee asset
SetTopic(message id)
```

### Requirements

- **Barrier.** `AllowTopLevelPaidExecutionFrom` must allow Asset Hub. The leading `ReserveAssetDeposited` holds one asset, below the `MAX_ASSETS_FOR_BUY_EXECUTION` limit of 2, and is followed by `PayFees`.
- **Trader.** Must accept Ether in `PayFees`, or DOT if the DOT-fee option is offered.
- **Reserves and teleports:**
  - ERC-20 and Ether: Asset Hub must be accepted as their reserve.
  - A PNA native to this chain arrives as `ReceiveTeleportedAsset` from Asset Hub, so Asset Hub must be a trusted teleporter for that asset.
- **Leftover fee deposit.** The leftover-fee `DepositAsset` must not trap funds. The SDK pads the fee so the leftover stays above the destination's Ether minimum balance. A minimum balance that is too high, or that differs from the registry, will trap the deposit.
- **Origin.** Plain transfers arrive with the origin cleared, so no aliasing is needed. Transact or contract-call flows that keep an Ethereum origin need the chain to let Asset Hub alias Ethereum origins (see [Parachain Integration](parachain-integration.md#generalised-message-passing)).

### SDK calls against the parachain

`query_xcm_weight` and `query_weight_to_asset_fee` (Ether or DOT) on the destination XCM. `dry_run_xcm` with origin Asset Hub, on the message forwarded by Asset Hub's dry-run. `system.account` and the token balance of the beneficiary, for the account-existence and max-consumer checks.

## 3. Parachain → Ethereum

The parachain **executes** a user-signed XCM program and forwards it to Asset Hub with the user's origin preserved. This direction needs the most from the chain.

### Extrinsic

`polkadotXcm.execute(message: VersionedXcm (V5), max_weight)`, signed by the user.

- `ExecuteXcmOrigin` must map a signed origin to `AccountId32` or `AccountKey20`.
- **`XcmExecuteFilter` must allow the program below for ordinary signed accounts.** If it doesn't, the call fails with `polkadotXcm.Filtered` (pallet-xcm error index 2).
- `transferAssetsUsingTypeAndThen` **cannot** be used for V2. pallet-xcm always inserts `ClearOrigin` before the custom XCM on Asset Hub. Without an origin, Asset Hub forwards `ClearOrigin` instead of `AliasOrigin`, and Bridge Hub's V2 converter then rejects the message with `AliasOriginExpected`. Chains that only allow `transferAssetsUsingTypeAndThen` can still send to Ethereum over **V1** (see [Enabling](#enabling)).

### Program executed locally (default: DOT pays fees)

```
WithdrawAsset([DOT, Ether, token])   // token omitted when sending Ether
PayFees(DOT)                          // local execution + local delivery
SetAppendix([RefundSurplus, DepositAsset(AllCounted(3), user)])
InitiateTransfer {
  destination: Asset Hub,
  remote_fees: ReserveWithdraw(DOT),
  preserve_origin: true,              // → AliasOrigin(Parachain(id)/AccountId32(user)) on Asset Hub
  assets: [ReserveWithdraw(Ether), ReserveWithdraw(token)],   // PNA native to this chain: Teleport(token)
  remote_xcm: <Asset Hub program>,
}
SetTopic(message id)
```

The fee-asset options change this program:
- **DOT-as-fee** (`feeTokenLocation = DOT`): Ether is bought on Asset Hub with `ExchangeAsset`, so the chain never withdraws Ether.
- **Native-as-fee:** the chain's native token is teleported to Asset Hub and swapped there.

### Requirements

- **Executor.** Must support `WithdrawAsset`, `PayFees`, `SetAppendix`, `RefundSurplus`, `DepositAsset`, `InitiateTransfer` with `preserve_origin: true`, and `SetTopic`.
- **Withdrawals.** Must be able to withdraw DOT, Ether, ERC-20s and, for PNAs, the native asset in the leading `WithdrawAsset`. Any custom barrier, filter or asset-transactor code that inspects that instruction must handle every one of these without panicking.
- **Reserves and teleports:**
  - DOT, Ether and ERC-20s are reserve-withdrawn to Asset Hub.
  - A PNA native to this chain is teleported, so it must be teleportable to Asset Hub.
  - On Asset Hub that needs the foreign asset's owner to have configured both Asset Hub and this chain as trusted reserves (`TeleportableAssetWithTrustedReserve`).
- **`XcmPaymentApi.query_delivery_fees(Asset Hub, forwarded XCM)`** must be implemented:
  - The SDK calls it with two arguments (API v1) and falls back to three, with DOT as the asset (API v2).
  - The result must contain DOT (`{ parents: 1, Here }`), or the native token when it is the fee asset.
  - A result that contains neither is treated as 0 with a warning. `Unimplemented` aborts the quote.
- **Account location.** The user's location on Asset Hub, `Parachain(id)/AccountId32` or `AccountKey20`, is a child of the chain's own location. Asset Hub's `AliasChildLocation` therefore accepts the alias without governance action. Bridge Hub derives the Ethereum agent from that same origin.
- **Funded accounts.** The user needs DOT for all Polkadot-side fees, Ether for Ethereum execution plus any service fee, and the token itself. The chain's sovereign account on Asset Hub must hold matching DOT, Ether and ERC-20 reserves.

### SDK calls against the parachain

- `query_xcm_weight` on the local program. The result is padded by 25% and capped at the chain's max weight; this becomes `max_weight` for `execute`.
- `query_weight_to_asset_fee(DOT)`, or `(native)` for chains without DOT balances.
- `query_delivery_fees(Asset Hub, ..)`.
- `dry_run_call` with origin `Signed(user)`. It passes `result_xcms_version = 4` and retries without it. The result must include the XCM forwarded to Asset Hub, under a V4 or V5 location.
- Balance reads for native, DOT, Ether and the token.
- `paymentInfo` on the extrinsic.

## 4. Polkadot-native assets (PNAs) from the parachain

For an asset whose reserve is this parachain:

1. Create it as a foreign asset on Asset Hub, with reserves configured so teleports between Asset Hub and the chain are trusted both ways.
2. Register it on Bridge Hub (`EthereumSystem` / `EthereumSystemV2` `register_token`). The Ethereum token ID is `TokenIdOf` of `GlobalConsensus(Polkadot)/Parachain(id)/<local junctions>`. V1 and V2 exports derive it the same way.
3. The chain's asset registry must map `{ parents: 0, interior: <local junctions> }` to the asset, as the location the SDK uses on the chain. The SDK records three locations for each PNA in the registry:
   - `location`: local to the chain;
   - `locationOnAH`: `{ parents: 1, X(Parachain(id), ..) }`;
   - `locationOnEthereum`.

No Gateway approval is needed on Ethereum: the Gateway burns PNAs directly from the sender.

## 5. SDK and registry integration

These are changes on the Snowbridge side, but each chain needs them:

- **A `ParachainBase` implementation for the runtime's `specName`** (`web/packages/api/src/parachains/`), registered in `parachains/index.ts`. It implements:
  - `getAssets`, which discovers Ether, ERC-20s and PNAs and returns their locations, decimals and minimum balances;
  - `getLocationBalance`, including native-token balances when a PNA is the native token;
  - `getDotBalance` and `getXC20DOT`.
- **Registry environment** (`registry/scripts/buildRegistry.ts`):
  - an RPC endpoint under `parachains`;
  - the parachain ID in `v2_parachains` (both directions) **or** `v2_from_ethereum_parachains` (Ethereum → parachain on V2, parachain → Ethereum on V1);
  - route filters for any asset that should only travel to particular chains.
- **Feature gates computed by the registry builder** (`Parachain.features`):

  | Direction | Required features |
  |---|---|
  | Ethereum → parachain on V2 | `hasXcmPaymentApi`, `xcmVersion === "v5"`, and `supportsV2` or `supportsV2FromEthereum` |
  | Parachain → Ethereum on V2 | the above, plus `hasEthBalance` and `supportsAliasOrigin` |

  `hasDryRunApi` and `hasDotBalance` select validation behaviour and the fee path.
- **dApp:** token and chain icons at `public/images/<symbol>.png` and `public/images/polkadot_<id>.png`. The dApp reads `features.supportsV2` to choose V2 fee options for parachain → Ethereum.

## Enabling

| Registry entry | Ethereum → parachain | Parachain → Ethereum |
|---|---|---|
| neither | V1 (no PNAs to parachains) | V1 via `transferAssetsUsingTypeAndThen` |
| `v2_from_ethereum_parachains` | V2 | V1 via `transferAssetsUsingTypeAndThen` |
| `v2_parachains` | V2 | V2 via `polkadotXcm.execute` |

Use `v2_from_ethereum_parachains` while a chain satisfies Sections 1–2 but not Section 3, for example while `execute` is filtered or `query_delivery_fees` is unimplemented.

## Verifying a chain

Every requirement above can be checked against a live chain without signing anything:

1. `polkadotXcm.supportedVersion` on the chain and on Asset Hub shows v5 for each other.
2. `query_xcm_weight` on single-instruction v5 programs (`AliasOrigin`, `PayFees`, `InitiateTransfer`, `SetHints`) returns finite weights.
3. `query_weight_to_asset_fee` succeeds for DOT and Ether.
4. `query_delivery_fees(Asset Hub, ..)` returns `Ok` with a DOT asset.
5. `dry_run_call(Signed(user), polkadotXcm.execute(<program above>))` returns `Ok`, not `Filtered` and not a runtime panic, for each asset type (Ether, ERC-20, native PNA).
6. Feed the forwarded XCM into `dry_run_xcm` on Asset Hub, with origin the parachain, and then on Bridge Hub, with origin Asset Hub. Bridge Hub must emit `ethereumOutboundQueueV2.MessageQueued` with the expected `UnlockNativeToken` or `MintForeignToken` command.
7. For Ethereum → parachain, the SDK's `validate()` dry-runs Asset Hub and the destination end to end.

## Common failure modes

| Symptom | Cause |
|---|---|
| `polkadotXcm.Filtered` from `execute` | `XcmExecuteFilter` does not allow user programs |
| Quote fails with "Can not query XCM Weight" on parachain → Ethereum | `query_delivery_fees` returns `Unimplemented` (the SDK reuses the weight error message) |
| `UntrustedReserveLocation` on Asset Hub, Ethereum → parachain | A parachain-native PNA was sent as a reserve transfer; it must be teleported |
| `AliasOriginExpected` on Bridge Hub | The program reached Asset Hub without an origin (for example, sent via `transferAssetsUsingTypeAndThen`) |
| Runtime panic (`wasm unreachable`) when the native asset is in the leading `WithdrawAsset` | Chain-specific barrier, filter or transactor code that cannot handle its native asset there; the chain must fix it |
| `Barrier` in `dry_run_xcm` on the source with more than 2 withdrawn assets | Artifact of `dry_run_xcm`, which has no weight credit; real `execute` passes `TakeWeightCredit`. Use `dry_run_call` |
