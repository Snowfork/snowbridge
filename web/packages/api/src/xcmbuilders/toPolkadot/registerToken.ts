import { erc20Location, ethereumNetwork, bridgeLocation } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { u8aToHex } from "@polkadot/util"
import { blake2AsU8a } from "@polkadot/util-crypto"
import { ApiPromise } from "@polkadot/api"

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
export function getBridgeOwnerAccount(assetHub: any, ethChainId: number): string {
    const registry = assetHub.registry

    // Encode "ethereum-chain" as bytes
    const prefix = new TextEncoder().encode("ethereum-chain")

    // Encode chain_id as u64 (little-endian)
    const chainIdBytes = new Uint8Array(8)
    const view = new DataView(chainIdBytes.buffer)
    view.setBigUint64(0, BigInt(ethChainId), true) // true = little-endian

    // Concatenate and hash
    const combined = new Uint8Array([...prefix, ...chainIdBytes])
    const hash = blake2AsU8a(combined, 256)

    const bridgeOwner = u8aToHex(hash)
    console.log("Bridge Owner Account:", bridgeOwner)
    console.log("Chain ID:", ethChainId)

    return bridgeOwner
}

/**
 * Build the remote XCM for token registration that will be executed on AssetHub
 * This XCM creates a new foreign asset in AssetHub's ForeignAssets pallet
 *
 * Based on make_create_asset_xcm_for_polkadot in gas-estimator/src/xcm_builder.rs:317-361
 */
export function buildRegisterTokenXcm(
    assetHub: ApiPromise,
    ethChainId: number,
    tokenAddress: string,
    ethAmount: bigint,
    claimer: any,
    bridgeOwner: string,
    assetDepositDOT: bigint
) {
    const registry = assetHub.registry

    const dotAsset = {
        parents: 1,
        interior: { Here: null },
    }

    const ethAsset = {
        id: bridgeLocation(ethChainId),
        fun: {
            Fungible: ethAmount,
        },
    }

    const dotFeeAsset = {
        id: dotAsset,
        fun: {
            Fungible: assetDepositDOT,
        },
    }

    // Asset ID for the token being registered
    const assetIdLocation =
        tokenAddress === ETHER_TOKEN_ADDRESS
            ? bridgeLocation(ethChainId)
            : erc20Location(ethChainId, tokenAddress)

    // Create the ForeignAssets::create call using the API
    const assetIdLocationTyped = registry.createType("StagingXcmV5Location", assetIdLocation)
    const adminMultiAddress = registry.createType("MultiAddress", { Id: bridgeOwner })

    // Use the API to create the call properly
    const createCall = assetHub.tx.foreignAssets.create(
        assetIdLocationTyped,
        adminMultiAddress,
        MINIMUM_DEPOSIT
    )

    const callData = createCall.method.toU8a()
    console.log("assetDepositDOT", assetDepositDOT)

    return registry.createType("XcmVersionedXcm", {
        V5: [
            {
                ExchangeAsset: {
                    give: { Definite: [ethAsset] },
                    want: [dotFeeAsset],
                    maximal: false,
                },
            },
            {
                DepositAsset: {
                    assets: { Definite: [dotFeeAsset] },
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
                    beneficiary: claimer,
                },
            },
        ],
    })
}

/**
 * Build the full XCM message that will be executed on AssetHub for dry running
 * This mimics what the relayer builds when processing a registration message
 *
 * Based on build_asset_hub_xcm in gas-estimator/src/xcm_builder.rs:52-130
 */
export function buildAssetHubRegisterTokenXcm(
    assetHub: ApiPromise,
    ethChainId: number,
    totalValue: bigint,
    executionFee: bigint,
    claimer: any,
    origin: string,
    remoteXcmBytes: Uint8Array
) {
    const registry = assetHub.registry
    const ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)

    // Decode the remote XCM
    const remoteXcm = registry.createType("XcmVersionedXcm", remoteXcmBytes) as any
    const remoteInstructions = remoteXcm.isV5 ? remoteXcm.asV5 : []

    const instructions = [
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
                hints: [{ AssetClaimer: { location: claimer } }],
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

    // Add remaining ETH if any
    if (totalValue > executionFee) {
        instructions.push({
            ReserveAssetDeposited: [
                {
                    id: ether,
                    fun: {
                        Fungible: totalValue - executionFee,
                    },
                },
            ],
        })
    }

    // Add descend origin for sender
    instructions.push({
        DescendOrigin: {
            X1: [
                {
                    AccountKey20: {
                        key: origin,
                        network: null,
                    },
                } as any,
            ],
        },
    } as any)

    // Append remote XCM instructions
    instructions.push(...remoteInstructions)

    return registry.createType("XcmVersionedXcm", {
        V5: instructions,
    })
}
