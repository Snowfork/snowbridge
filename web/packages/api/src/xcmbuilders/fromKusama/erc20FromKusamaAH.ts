import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    accountToLocation,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"

/**
 * Builds the customXcm that executes on Polkadot AssetHub after receiving assets from Kusama AH.
 *
 * This XCM:
 * 1. Pays fees in DOT for AH execution
 * 2. Swaps DOT → Ether via exchangeAsset for Ethereum execution fee
 * 3. Uses initiateTransfer to forward to Ethereum via Snowbridge
 *
 * Flow: Kusama AH → (Polkadot↔Kusama bridge) → Polkadot AH [this XCM] → Snowbridge → Ethereum
 */
export function buildPolkadotAHCustomXcm(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    localDOTFee: bigint,
    remoteEtherFee: bigint,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = erc20Location(ethChainId, asset.token)

    let remoteXcm = buildEthereumInstructions(beneficiaryLocation, topic, callHex)

    let instructions: any[] = [
        {
            payFees: {
                asset: {
                    id: DOT_LOCATION,
                    fun: {
                        Fungible: localDOTFee,
                    },
                },
            },
        },
        {
            setAppendix: [
                {
                    refundSurplus: null,
                },
                {
                    depositAsset: {
                        assets: {
                            wild: {
                                allCounted: 3,
                            },
                        },
                        beneficiary: {
                            parents: 0,
                            interior: { x1: [sourceLocation] },
                        },
                    },
                },
            ],
        },
        {
            exchangeAsset: {
                give: {
                    Wild: {
                        AllOf: {
                            id: DOT_LOCATION,
                            fun: "Fungible",
                        },
                    },
                },
                want: [
                    {
                        id: bridgeLocation(ethChainId),
                        fun: {
                            Fungible: remoteEtherFee,
                        },
                    },
                ],
                maximal: false,
            },
        },
        {
            initiateTransfer: {
                destination: bridgeLocation(ethChainId),
                remote_fees: {
                    reserveWithdraw: {
                        definite: [
                            {
                                id: bridgeLocation(ethChainId),
                                fun: {
                                    Fungible: remoteEtherFee,
                                },
                            },
                        ],
                    },
                },
                preserveOrigin: true,
                assets: [
                    {
                        reserveWithdraw: {
                            definite: [
                                {
                                    id: tokenLocation,
                                    fun: {
                                        Fungible: tokenAmount,
                                    },
                                },
                            ],
                        },
                    },
                ],
                remoteXcm: remoteXcm,
            },
        },
        {
            setTopic: topic,
        },
    ]
    return registry.createType("XcmVersionedXcm", {
        v5: instructions,
    })
}
