import {accountToLocation, bridgeLocation, erc20Location, ethereumNetwork} from "../../xcmBuilder"
import {ETHER_TOKEN_ADDRESS} from "../../assets_v2"
import {u8aToHex} from "@polkadot/util"
import {blake2AsU8a} from "@polkadot/util-crypto"
import {ApiPromise} from "@polkadot/api"
import {claimerFromBeneficiary} from "../../toPolkadotSnowbridgeV2";

// Constants from gas-estimator/src/config.rs
export const MINIMUM_DEPOSIT = 1n

/**
 * Get the sovereign account of Ethereum on Asset Hub
 * This is the "bridge owner" account that owns all bridged assets
 *
 * Uses ExternalConsensusLocationsConverterFor logic:
 * blake2_256(encode("ethereum-chain", chain_id))
 *
 * Based on staging-xcm-builder/src/location_conversion.rs
 */
export function getBridgeOwnerAccount(ethChainId: number): string {
    const prefix = new TextEncoder().encode("ethereum-chain")
    const chainIdBytes = new Uint8Array(8)
    const view = new DataView(chainIdBytes.buffer)
    view.setBigUint64(0, BigInt(ethChainId), true) // true = little-endian

    const combined = new Uint8Array([...prefix, ...chainIdBytes])
    const hash = blake2AsU8a(combined, 256)

    return u8aToHex(hash)
}

/**
 * Build the full XCM message for token registration that will be executed on AssetHub
 * This combines the BridgeHub wrapper and the remote registration XCM
 *
 * Based on:
 * - build_asset_hub_xcm in gas-estimator/src/xcm_builder.rs:52-130
 * - make_create_asset_xcm_for_polkadot in gas-estimator/src/xcm_builder.rs:317-361
 */
export function buildAssetHubRegisterTokenXcm(
    assetHub: ApiPromise,
    ethChainId: number,
    tokenAddress: string,
    totalValue: bigint,
    executionFee: bigint,
    assetDepositDOT: bigint,
    bridgeOwner: string
) {
    const registry = assetHub.registry
    const ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)

    // Calculate remaining ETH for registration after execution fee
    const ethAmountForRegistration = totalValue - executionFee

    const assetIdLocation =
        tokenAddress === ETHER_TOKEN_ADDRESS
            ? bridgeLocation(ethChainId)
            : erc20Location(ethChainId, tokenAddress)

    const assetIdLocationTyped = registry.createType("StagingXcmV5Location", assetIdLocation)
    const adminMultiAddress = registry.createType("MultiAddress", { Id: bridgeOwner })

    const createCall = assetHub.tx.foreignAssets.create(
        assetIdLocationTyped,
        adminMultiAddress,
        MINIMUM_DEPOSIT
    )

    const callData = createCall.method.toU8a()
    let bridgeOwnerLocation = accountToLocation(bridgeOwner)

    const instructions: any[] = [
        {
            DescendOrigin: { X1: [{ PalletInstance: 91 }] },
        },
        {
            UniversalOrigin: ethereumNetwork(ethChainId),
        },
        {
            ReserveAssetDeposited: [
                {
                    id: ether,
                    fun: {
                        Fungible: executionFee,
                    },
                },
            ],
        },
        {
            SetHints: {
                hints: [{ AssetClaimer: { location: claimerFromBeneficiary(assetHub, bridgeOwner) } }],
            },
        },
        {
            PayFees: {
                asset: {
                    id: ether,
                    fun: {
                        Fungible: executionFee,
                    },
                },
            },
        },
    ]

    if (ethAmountForRegistration > 0n) {
        instructions.push({
            ReserveAssetDeposited: [
                {
                    id: ether,
                    fun: {
                        Fungible: ethAmountForRegistration,
                    },
                },
            ],
        })
    }

    instructions.push(
        {
            ExchangeAsset: {
                give: {
                    Definite: [
                        {
                            id: bridgeLocation(ethChainId),
                            fun: {
                                Fungible: ethAmountForRegistration,
                            },
                        },
                    ],
                },
                want: [
                    {
                        id: { parents: 1, interior: { Here: null } },
                        fun: {
                            Fungible: assetDepositDOT,
                        },
                    },
                ],
                maximal: true,
            },
        },
        {
            DepositAsset: {
                assets: {
                    Definite: [
                        {
                            id: { parents: 1, interior: { Here: null } },
                            fun: {
                                Fungible: assetDepositDOT,
                            },
                        },
                    ],
                },
                beneficiary: {
                    parents: 0,
                    interior: { x1: [bridgeOwnerLocation] }
                },
            },
        },
        {
            Transact: {
                originKind: "Xcm",
                fallbackMaxWeight: null,
                call: {
                    encoded: u8aToHex(callData),
                },
            },
        },
        {
            RefundSurplus: null,
        },
        {
            DepositAsset: {
                assets: {
                    Wild: {
                        AllCounted: 2,
                    },
                },
                beneficiary: {
                    parents: 0,
                    interior: {
                        X1: [
                            {
                                AccountId32: {
                                    network: null,
                                    id: bridgeOwner,
                                },
                            },
                        ],
                    },
                },
            },
        }
    )

    return registry.createType("XcmVersionedXcm", {
        V5: instructions,
    })
}
