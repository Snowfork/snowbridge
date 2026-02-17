# V1 to V2 Upgrade Guide

## Snowbridge V1 to V2 Migration Guide

This guide is for parachain teams (e.g. Hydration) migrating their Snowbridge integration from V1 to V2. It covers both transfer directions: Ethereum to Polkadot and Polkadot to Ethereum.

### Prerequisites

Your parachain must support:

* **XCM v5** (locations use array format: `x1: [junction]`)
* **XcmPaymentApi** runtime API (for fee estimation)
* **supportsV2** flag in the Snowbridge asset registry

For Polkadot-to-Ethereum V2 transfers, additionally:

* **Ether balance support** (`hasEthBalance`)
* **AliasOrigin support** (`supportsAliasOrigin`)

***

### 1. Ethereum to Polkadot

V2 example script: [https://github.com/Snowfork/snowbridge/blob/main/web/packages/operations/src/transfer\_to\_polkadot\_v2.ts](../../web/packages/operations/src/transfer_to_polkadot_v2.ts)

#### V1: `sendToken`

In V1, the Ethereum Gateway exposes `sendToken` which takes the token address, destination parachain ID, beneficiary, and fee amount. The bridge handles all XCM construction internally.

#### V2: `v2_sendMessage`

In V2, the caller constructs an XCM program and passes it to the gateway along with asset and fee parameters.

```solidity
function v2_sendMessage(
    bytes calldata xcm,          // SCALE-encoded VersionedXcm (your XCM program)
    bytes[] calldata assets,     // Array of ABI-encoded asset specs
    bytes calldata claimer,      // SCALE-encoded claimer location
    uint128 executionFee,        // Ether for AssetHub execution
    uint128 relayerFee           // Ether for relayer incentive
) external payable;
```

* `msg.value` must be >= `executionFee + relayerFee`. Any surplus becomes the transfer value (deposited as Ether into the XCM holding register on AssetHub).

#### XCM Location Constants

All XCM examples below use these locations:

```typescript
// Ether location (zero-address ERC20 = native Ether)
const ETHER_LOCATION = {
    parents: 2,
    interior: { x1: [{ GlobalConsensus: { Ethereum: { chain_id: 1 } } }] }
}

// ERC20 token location
const ERC20_LOCATION = {
    parents: 2,
    interior: {
        X2: [
            { GlobalConsensus: { Ethereum: { chain_id: 1 } } },
            { AccountKey20: { key: TOKEN_ADDRESS } }
        ]
    }
}

// DOT location
const DOT_LOCATION = { parents: 1, interior: "Here" }
```

Note: `ETHER_LOCATION` and `BRIDGE_LOCATION` are the same value (the Ethereum consensus location). The Ether token address is `0x0000000000000000000000000000000000000000`.

#### How `v2_sendMessage` Works: Infrastructure vs User XCM

**Important:** The `xcm` parameter you pass to `v2_sendMessage` is **not** the full XCM program that executes on AssetHub. BridgeHub's inbound pallet **prepends** infrastructure instructions before appending your XCM.

**BridgeHub prepends (you do NOT construct these):**

```
1. DescendOrigin(PalletInstance(91))              -- Snowbridge inbound pallet origin
2. UniversalOrigin(Ethereum { chain_id })          -- Establishes Ethereum origin
3. ReserveAssetDeposited([ETH executionFee])       -- Brings execution fee into holding
4. SetHints { assetClaimer: claimer }              -- From your `claimer` param
5. PayFees { ETH executionFee }                    -- Pays AH execution fees
6. ReserveAssetDeposited([ETH surplus value])      -- Ether value (msg.value - fees), if any
7. ReserveAssetDeposited([ERC20 tokens])           -- From your `assets` param (or WithdrawAsset for PNA)
8. DescendOrigin(AccountKey20 { msg.sender })      -- Sets sender sub-origin
```

**Then your `xcm` parameter is appended.** So what you pass as `xcm` only needs to handle the assets that are already in the holding register.

#### Encoding the `assets` Parameter

ERC20 tokens are ABI-encoded for the `assets` parameter:

```typescript
import { AbiCoder } from "ethers"

// For ERC20 tokens (NativeTokenERC20, kind=0):
const encodedAsset = AbiCoder.defaultAbiCoder().encode(
    ["uint8", "address", "uint128"],
    [0, tokenAddress, amount],
)

// Wrap into bytes[] for the gateway
const assetsBytes = AbiCoder.defaultAbiCoder().encode(
    ["bytes[]"],
    [[encodedAsset]],
)
```

Ether is **not** encoded as an asset. Instead, it is sent as surplus `msg.value` beyond `executionFee + relayerFee`. The surplus automatically appears in the holding register on AssetHub.

Example of encoding in TypeScript: [https://github.com/Snowfork/snowbridge/blob/main/web/packages/api/src/transfers/toPolkadot/erc20ToParachain.ts#L225-L229](../../web/packages/api/src/transfers/toPolkadot/erc20ToParachain.ts#L225-L229)

#### Encoding the `claimer` Parameter

The claimer is a SCALE-encoded `StagingXcmV5Location` identifying who can reclaim trapped assets on AssetHub:

```typescript
import { toPolkadotSnowbridgeV2 } from "@snowbridge/api"

const claimerLocation = toPolkadotSnowbridgeV2.claimerFromBeneficiary(
    assetHub, // ApiPromise
    beneficiaryAddressHex,
)
const claimerBytes = toPolkadotSnowbridgeV2.claimerLocationToBytes(claimerLocation)
```

#### Message ID (Topic)

V2 generates a unique topic for each transfer using blake2:

```typescript
const topic = toPolkadotSnowbridgeV2.buildMessageId(
    destParaId, senderHex, tokenAddress,
    beneficiary, amount, accountNonce,
)
```

#### `xcm` Parameter: ERC20 to Parachain (Ether Destination Fee)

For transfers to your parachain paying destination fees in **Ether**:

```
v5: [
    // Forward token + ether fee to destination parachain
    { initiateTransfer: {
        destination: { parents: 1, interior: { x1: [{ parachain: DEST_PARA_ID }] } },
        remote_fees: {
            reserveDeposit: {
                definite: [{ id: ETHER_LOCATION, fun: { Fungible: remoteExecutionFee } }]
            }
        },
        preserveOrigin: false,
        assets: [{
            reserveDeposit: {
                definite: [{ id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }]
            }
        }],
        // XCM that executes on the destination parachain:
        remoteXcm: [
            { refundSurplus: null },
            { depositAsset: {
                assets: { wild: { allCounted: 3 } },
                beneficiary: { parents: 0, interior: { x1: [BENEFICIARY_LOCATION] } }
            }},
            { setTopic: TOPIC }
        ]
    }},

    // Return any unused Ether fees on AssetHub to beneficiary
    { refundSurplus: null },
    { depositAsset: {
        assets: { wild: { allOf: { id: ETHER_LOCATION, fun: "Fungible" } } },
        beneficiary: { parents: 0, interior: { x1: [BENEFICIARY_LOCATION] } }
    }},
    { setTopic: TOPIC }
]
```

#### `xcm` Parameter: ERC20 to Parachain (DOT Destination Fee)

If your parachain prefers DOT as the destination fee asset, add `exchangeAsset` before `initiateTransfer` to swap Ether for DOT on AssetHub:

```
v5: [
    // Swap Ether for DOT on AssetHub (for destination fee)
    { exchangeAsset: {
        give: {
            definite: [{ id: ETHER_LOCATION, fun: { Fungible: remoteEtherFeeAmount } }]
        },
        want: [{ id: DOT_LOCATION, fun: { Fungible: remoteDotFeeAmount } }],
        maximal: true
    }},

    // Forward to destination with DOT as fee
    { initiateTransfer: {
        destination: { parents: 1, interior: { x1: [{ parachain: DEST_PARA_ID }] } },
        remote_fees: {
            reserveDeposit: {
                definite: [{ id: DOT_LOCATION, fun: { Fungible: remoteDotFeeAmount } }]
            }
        },
        preserveOrigin: false,
        assets: [{
            reserveDeposit: {
                definite: [{ id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }]
            }
        }],
        remoteXcm: [
            { refundSurplus: null },
            { depositAsset: {
                assets: { wild: { allCounted: 3 } },
                beneficiary: { parents: 0, interior: { x1: [BENEFICIARY_LOCATION] } }
            }},
            { setTopic: TOPIC }
        ]
    }},

    // Return unused Ether to beneficiary on AssetHub
    { depositAsset: {
        assets: { wild: { allOf: { id: ETHER_LOCATION, fun: "Fungible" } } },
        beneficiary: { parents: 0, interior: { x1: [BENEFICIARY_LOCATION] } }
    }},
    { setTopic: TOPIC }
]
```

#### `xcm` Parameter: PNA to Parachain

For Polkadot Native Assets (like DOT), the `xcm` parameter structure is identical to the ERC20 versions above. The difference is only in the infrastructure part: BridgeHub uses `WithdrawAsset` instead of `ReserveAssetDeposited` for PNA tokens (since they are held in reserve on AssetHub). Your `xcm` parameter still uses `reserveDeposit` in `initiateTransfer.assets` because from the destination parachain's perspective, AssetHub is the reserve.

See `web/packages/api/src/xcmbuilders/toPolkadot/pnaToParachain.ts` for the exact implementation.

#### Full XCM on AssetHub (for reference)

For completeness, the full XCM that actually executes on AssetHub is the infrastructure prefix + your `xcm` appended. For an **ERC20 to Parachain with Ether fees**:

```
[BridgeHub prepends]
 1. DescendOrigin(PalletInstance(91))
 2. UniversalOrigin(Ethereum { chain_id })
 3. ReserveAssetDeposited([ETH executionFee])
 4. SetHints { assetClaimer: claimer }
 5. PayFees { ETH executionFee }
 6. ReserveAssetDeposited([ETH remaining value])
 7. ReserveAssetDeposited([ERC20 tokenAmount])
 8. DescendOrigin(AccountKey20 { msg.sender })

[Your xcm parameter]
 9. InitiateTransfer { destination, remote_fees, assets, remoteXcm }
10. RefundSurplus
11. DepositAsset { leftover ether -> beneficiary }
12. SetTopic
```

This full form is what the `buildAssetHubERC20ReceivedXcm` functions construct for dry-run validation purposes.

#### Ethereum to Polkadot Fees

The `DeliveryFee` returned by `getDeliveryFee` contains:

| Field                          | Description                                               |
| ------------------------------ | --------------------------------------------------------- |
| `assetHubDeliveryFeeEther`     | BridgeHub to AssetHub delivery fee (in Ether)             |
| `assetHubExecutionFeeEther`    | AssetHub XCM execution fee (in Ether)                     |
| `destinationDeliveryFeeEther`  | AssetHub to destination parachain delivery fee (in Ether) |
| `destinationExecutionFeeEther` | Destination execution fee (in Ether, if Ether fee path)   |
| `destinationExecutionFeeDOT`   | Destination execution fee (in DOT, if DOT fee path)       |
| `relayerFee`                   | Relayer incentive (in Ether)                              |
| `totalFeeInWei`                | Sum of all fees -- this is `msg.value` to send            |

The `totalFeeInWei` is split into the `v2_sendMessage` params as:

* `executionFee` = `assetHubExecutionFeeEther + destinationDeliveryFeeEther` (+ swap amount if DOT path)
* `relayerFee` = relayer incentive

All fees are padded by **33%** to account for weight estimation variance. Exchange rate swaps are padded by an additional **20%** slippage.

#### Using the TypeScript API (Ethereum to Polkadot)

```typescript
import {
    Context,
    toPolkadotSnowbridgeV2,
    xcmBuilder,
} from "@snowbridge/api"
import { bridgeInfoFor } from "@snowbridge/registry"

const { registry, environment } = bridgeInfoFor("polkadot_mainnet")
const context = new Context(environment)

// 1. Create transfer implementation (automatically selects ERC20 vs PNA)
const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
    destParaId,
    registry,
    TOKEN_ADDRESS,
)

// 2. Get delivery fee (optional: specify DOT as fee asset)
const fee = await transferImpl.getDeliveryFee(
    context, registry, TOKEN_ADDRESS, destParaId,
    { feeAsset: xcmBuilder.DOT_LOCATION }  // omit for Ether fees
)

// 3. Build the transfer transaction
const transfer = await transferImpl.createTransfer(
    {
        ethereum: context.ethereum(),
        assetHub: await context.assetHub(),
        destination: await context.parachain(destParaId),
    },
    registry, destParaId,
    ETHEREUM_SENDER, POLKADOT_BENEFICIARY,
    TOKEN_ADDRESS, amount, fee,
)

// 4. Validate (dry run on AH and BridgeHub)
const validation = await transferImpl.validateTransfer(
    {
        ethereum: context.ethereum(),
        gateway: context.gatewayV2(),
        bridgeHub: await context.bridgeHub(),
        assetHub: await context.assetHub(),
        destination: await context.parachain(destParaId),
    },
    transfer,
)

// 5. Send
const response = await wallet.sendTransaction(transfer.tx)
const receipt = await response.wait(1)
const message = await toPolkadotSnowbridgeV2.getMessageReceipt(receipt)
```

***

### 2. Polkadot to Ethereum

V2 example script: [https://github.com/Snowfork/snowbridge/blob/main/web/packages/operations/src/transfer\_to\_ethereum\_v2.ts](../../web/packages/operations/src/transfer_to_ethereum_v2.ts)

#### V1: `transfer_assets_using_reserve_type_and_then`

In V1, transfers from Polkadot to Ethereum use the `polkadotXcm.transfer_assets_using_reserve_type_and_then` extrinsic. The runtime constructs the XCM for you based on the provided parameters.

#### V2: XCM Execute

In V2, the transfer is done via `polkadotXcm.execute` with a manually constructed XCM program. The XCM withdraws the token and fees, then uses `initiateTransfer` to route through AssetHub to the Ethereum bridge.

#### XCM Location Constants

```typescript
// DOT location
const DOT_LOCATION = { parents: 1, interior: "Here" }

// Bridge/Ethereum location (also the Ether location)
const BRIDGE_LOCATION = {
    parents: 2,
    interior: { x1: [{ GlobalConsensus: { Ethereum: { chain_id: ETH_CHAIN_ID } } }] }
}

// ERC20 token location
const ERC20_LOCATION = {
    parents: 2,
    interior: {
        X2: [
            { GlobalConsensus: { Ethereum: { chain_id: ETH_CHAIN_ID } } },
            { AccountKey20: { key: TOKEN_ADDRESS } }
        ]
    }
}
```

#### ERC20 from Parachain (with Ether on source chain)

When the source parachain holds Ether (e.g. via the Snowbridge Ether foreign asset), the XCM uses Ether directly for the Ethereum execution fee:

```
v5: [
    // 1. Withdraw all needed assets on the source parachain
    { withdrawAsset: [
        { id: DOT_LOCATION, fun: { Fungible: totalDOTFee } },
        { id: BRIDGE_LOCATION, fun: { Fungible: ethereumExecutionFee } },
        { id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }
    ]},

    // 2. Pay local parachain execution fee in DOT
    { payFees: { asset: { id: DOT_LOCATION, fun: { Fungible: localDOTFee } } } },

    // 3. Error recovery on source chain: return assets to sender on failure
    { setAppendix: [
        { refundSurplus: null },
        { depositAsset: {
            assets: { wild: { allCounted: 3 } },
            beneficiary: { parents: 0, interior: { x1: [SENDER_LOCATION] } }
        }}
    ]},

    // 4. Forward everything to AssetHub
    { initiateTransfer: {
        destination: { parents: 1, interior: { x1: [{ parachain: ASSET_HUB_PARA_ID }] } },
        remote_fees: {
            reserveWithdraw: {
                definite: [{ id: DOT_LOCATION, fun: { Fungible: totalDOTFee - localDOTFee } }]
            }
        },
        preserveOrigin: true,
        assets: [
            // Ether for Ethereum execution fee
            { reserveWithdraw: {
                definite: [{ id: BRIDGE_LOCATION, fun: { Fungible: ethereumExecutionFee } }]
            }},
            // The ERC20 token
            { reserveWithdraw: {
                definite: [{ id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }]
            }}
        ],
        // XCM to execute on AssetHub:
        remoteXcm: [
            // Error recovery on AssetHub
            { setAppendix: APPENDIX_INSTRUCTIONS },

            // Forward from AssetHub to Ethereum
            { initiateTransfer: {
                destination: BRIDGE_LOCATION,
                remote_fees: {
                    reserveWithdraw: {
                        definite: [{ id: BRIDGE_LOCATION, fun: { Fungible: ethereumExecutionFee } }]
                    }
                },
                preserveOrigin: true,
                assets: [{
                    reserveWithdraw: {
                        definite: [{ id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }]
                    }
                }],
                remoteXcm: [
                    { depositAsset: {
                        assets: { wild: { allCounted: 3 } },
                        beneficiary: { parents: 0, interior: { x1: [{ AccountKey20: { key: ETH_BENEFICIARY } }] } }
                    }},
                    { setTopic: TOPIC }
                ]
            }},
            { setTopic: TOPIC }
        ]
    }},
    { setTopic: TOPIC }
]
```

#### ERC20 from Parachain (DOT-only fee, no Ether on source)

If your parachain does not hold Ether, you can pay the Ethereum execution fee in DOT by adding an `exchangeAsset` on AssetHub to swap DOT for Ether:

```
v5: [
    // 1. Withdraw DOT and the token (no Ether needed on source chain)
    { withdrawAsset: [
        { id: DOT_LOCATION, fun: { Fungible: totalDOTFee } },
        { id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }
    ]},

    // 2. Pay local fee in DOT
    { payFees: { asset: { id: DOT_LOCATION, fun: { Fungible: localDOTFee } } } },

    // 3. Error recovery on source chain
    { setAppendix: [
        { refundSurplus: null },
        { depositAsset: {
            assets: { wild: { allCounted: 3 } },
            beneficiary: { parents: 0, interior: { x1: [SENDER_LOCATION] } }
        }}
    ]},

    // 4. Forward to AssetHub
    { initiateTransfer: {
        destination: { parents: 1, interior: { x1: [{ parachain: ASSET_HUB_PARA_ID }] } },
        remote_fees: {
            reserveWithdraw: {
                definite: [{
                    id: DOT_LOCATION,
                    fun: { Fungible: totalDOTFee - localDOTFee - ethereumExecutionFeeInDOT }
                }]
            }
        },
        preserveOrigin: true,
        assets: [
            // DOT that will be swapped to Ether on AssetHub
            { reserveWithdraw: {
                definite: [{ id: DOT_LOCATION, fun: { Fungible: ethereumExecutionFeeInDOT } }]
            }},
            // The ERC20 token
            { reserveWithdraw: {
                definite: [{ id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }]
            }}
        ],
        // XCM to execute on AssetHub:
        remoteXcm: [
            // Error recovery on AssetHub
            { setAppendix: APPENDIX_INSTRUCTIONS },

            // Swap DOT for Ether on AssetHub
            { exchangeAsset: {
                give: { Wild: { AllOf: { id: DOT_LOCATION, fun: "Fungible" } } },
                want: [{ id: BRIDGE_LOCATION, fun: { Fungible: ethereumExecutionFee } }],
                maximal: false
            }},

            // Forward to Ethereum with Ether fee
            { initiateTransfer: {
                destination: BRIDGE_LOCATION,
                remote_fees: {
                    reserveWithdraw: {
                        definite: [{ id: BRIDGE_LOCATION, fun: { Fungible: ethereumExecutionFee } }]
                    }
                },
                preserveOrigin: true,
                assets: [{
                    reserveWithdraw: {
                        definite: [{ id: ERC20_TOKEN_LOCATION, fun: { Fungible: tokenAmount } }]
                    }
                }],
                remoteXcm: [
                    { depositAsset: {
                        assets: { wild: { allCounted: 3 } },
                        beneficiary: { parents: 0, interior: { x1: [{ AccountKey20: { key: ETH_BENEFICIARY } }] } }
                    }},
                    { setTopic: TOPIC }
                ]
            }},
            { setTopic: TOPIC }
        ]
    }},
    { setTopic: TOPIC }
]
```

#### PNA (Polkadot Native Asset) from Parachain

For PNA transfers, the differences from ERC20 are:

* The token uses `teleport` instead of `reserveWithdraw` when moving from source to AssetHub
* On AssetHub, the token uses `reserveDeposit` when forwarding to Ethereum (since the PNA is held in reserve on AssetHub, deposited to the bridge)

Both Ether-fee and DOT-fee variants are available. See `web/packages/api/src/xcmbuilders/toEthereum/pnaFromParachain.ts` and `pnaFromParachainWithDotAsFee.ts` for the exact implementations.

#### Error Recovery (Appendix Instructions)

V2 uses `setAppendix` for error recovery. The appendix XCM runs if any subsequent instruction fails. On AssetHub, the appendix typically:

1. Sets an `assetClaimer` hint (so the sender can reclaim trapped assets)
2. Refunds surplus fees
3. Deposits remaining assets back to the sender's parachain account

```typescript
// Built by buildAppendixInstructions() in xcmBuilder.ts
[
    { setHints: { hints: [{ assetClaimer: { location: claimerLocation } }] } },
    { refundSurplus: null },
    { depositAsset: {
        assets: { wild: { allCounted: 3 } },
        beneficiary: claimerLocation ?? {
            parents: 1,
            interior: { x2: [{ parachain: sourceParaId }, senderAccountLocation] }
        }
    }}
]
```

#### Polkadot to Ethereum Fees

The `DeliveryFee` returned by `getDeliveryFee` contains:

| Field                           | Description                                                                                |
| ------------------------------- | ------------------------------------------------------------------------------------------ |
| `localExecutionFeeDOT`          | Source parachain XCM execution fee                                                         |
| `localDeliveryFeeDOT`           | Source to AssetHub delivery fee                                                            |
| `assetHubExecutionFeeDOT`       | AssetHub XCM execution fee                                                                 |
| `bridgeHubDeliveryFeeDOT`       | AssetHub to BridgeHub delivery fee                                                         |
| `snowbridgeDeliveryFeeDOT`      | Snowbridge protocol fee (governance-set, read from `:BridgeHubEthereumBaseFeeV2:` storage) |
| `returnToSenderExecutionFeeDOT` | Error recovery XCM execution fee                                                           |
| `returnToSenderDeliveryFeeDOT`  | Error recovery delivery fee                                                                |
| `totalFeeInDot`                 | Sum of all DOT fees                                                                        |
| `ethereumExecutionFee`          | Ethereum gas cost (in Ether)                                                               |
| `ethereumExecutionFeeInNative`  | Ethereum cost converted to fee token (if DOT or native fee path)                           |

All fees are padded by **33%**. Exchange rate swaps are padded by an additional **20%** slippage.

#### Using the TypeScript API (Polkadot to Ethereum)

```typescript
import {
    Context,
    toEthereumSnowbridgeV2,
    toEthereumV2,
    xcmBuilder,
} from "@snowbridge/api"
import { bridgeInfoFor } from "@snowbridge/registry"

const { registry, environment } = bridgeInfoFor("polkadot_mainnet")
const context = new Context(environment)

// 1. Create transfer implementation (auto-selects ERC20/PNA)
const transferImpl = await toEthereumSnowbridgeV2.createTransferImplementation(
    sourceParaId,
    registry,
    TOKEN_ADDRESS,
)

// 2. Get delivery fee
// Option A: Pay Ethereum execution fee in DOT (swapped on AH)
const fee = await transferImpl.getDeliveryFee(
    { sourceParaId, context },
    registry, TOKEN_ADDRESS,
    { feeTokenLocation: xcmBuilder.DOT_LOCATION },
)
// Option B: Pay in parachain native token
const fee = await transferImpl.getDeliveryFee(
    { sourceParaId, context },
    registry, TOKEN_ADDRESS,
    { feeTokenLocation: xcmBuilder.parachainLocation(sourceParaId) },
)
// Option C: Pay Ethereum fee in Ether (requires Ether balance on source)
const fee = await transferImpl.getDeliveryFee(
    { sourceParaId, context },
    registry, TOKEN_ADDRESS,
)

// 3. Build transfer
const transfer = await transferImpl.createTransfer(
    { sourceParaId, context },
    registry,
    POLKADOT_SENDER, ETHEREUM_BENEFICIARY,
    TOKEN_ADDRESS, amount, fee,
)

// 4. Validate (dry runs on source, AH, BridgeHub)
const validation = await transferImpl.validateTransfer(context, transfer)

// 5. Sign and send
const response = await toEthereumSnowbridgeV2.signAndSend(
    context, transfer, polkadotAccount,
    { withSignedTransaction: true },
)
```

***

### Key Differences Summary

| Aspect                   | V1                                                        | V2                                                          |
| ------------------------ | --------------------------------------------------------- | ----------------------------------------------------------- |
| **Ethereum entry point** | `sendToken(token, paraId, beneficiary, fee)`              | `v2_sendMessage(xcm, assets, claimer, execFee, relayerFee)` |
| **Polkadot extrinsic**   | `polkadotXcm.transfer_assets_using_reserve_type_and_then` | `polkadotXcm.execute` with custom XCM                       |
| **XCM version**          | v4                                                        | v5                                                          |
| **Fee instruction**      | `buyExecution`                                            | `payFees`                                                   |
| **Transfer instruction** | `depositReserveAsset` / `initiateReserveWithdraw`         | `initiateTransfer`                                          |
| **AH fee asset (E2P)**   | DOT                                                       | Ether (with optional DOT for destination)                   |
| **Error recovery**       | Limited                                                   | `setAppendix` with claimer, refund, and deposit back        |
| **Custom XCM**           | Not supported                                             | `customXcm` parameter for extra instructions at destination |
| **Fee splitting (E2P)**  | Single fee                                                | Separate `executionFee` and `relayerFee`                    |

### Reference Files

* **E2P XCM builders**: [https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/xcmbuilders/toPolkadot/](https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/xcmbuilders/toPolkadot/)
* **P2E XCM builders**: [https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/xcmbuilders/toEthereum/](https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/xcmbuilders/toEthereum/)
* **E2P transfer implementations**: [https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/transfers/toPolkadot](https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/transfers/toPolkadot)
* **P2E transfer implementations**: [https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/transfers/toEthereum](https://github.com/Snowfork/snowbridge/tree/main/web/packages/api/src/transfers/toEthereum)
* **Gateway V2 Solidity interface**: [https://github.com/Snowfork/snowbridge/blob/main/contracts/src/v2/IGateway.sol](../../contracts/src/v2/IGateway.sol)

