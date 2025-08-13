import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    parachainLocation,
    accountToLocation,
    HERE_LOCATION,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"

export function buildTransferXcmFromParachainWithNativeAssetFee(
    registry: Registry,
    ethChainId: number,
    assetHubParaId: number,
    sourceParachainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    localNativeFeeAmount: bigint,
    totalNativeFeeAmount: bigint,
    remoteEtherFeeAmount: bigint,
    remoteEtherFeeNativeAmount: bigint,
    claimerLocation?: any
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = erc20Location(ethChainId, asset.token)
    let assets = [
        {
            id: HERE_LOCATION,
            fun: {
                Fungible: totalNativeFeeAmount,
            },
        },
        {
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        },
    ]

    claimerLocation = claimerLocation ?? {
        parents: 0,
        interior: { x1: [sourceLocation] },
    }
    let remoteInstructionsOnAH: any[] = [
        {
            setAppendix: [
                {
                    setHints: {
                        hints: [{ assetClaimer: { location: claimerLocation } }],
                    },
                },
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
                        beneficiary: claimerLocation,
                    },
                },
            ],
        },
        // The first swap native asset to DOT
        {
            exchangeAsset: {
                give: {
                    Wild: {
                        AllOf: {
                            id: {
                                parents: 1,
                                interior: { x1: [{ parachain: sourceParachainId }] },
                            },
                            fun: "Fungible",
                        },
                    },
                },
                want: [
                    {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: 1n,
                        },
                    },
                ],
                maximal: true,
            },
        },
        // The second swap DOT to Ether
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
                            Fungible: remoteEtherFeeAmount,
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
                                    Fungible: remoteEtherFeeAmount,
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
                remoteXcm: [
                    {
                        depositAsset: {
                            assets: {
                                wild: {
                                    allCounted: 3,
                                },
                            },
                            beneficiary: {
                                parents: 0,
                                interior: { x1: [beneficiaryLocation] },
                            },
                        },
                    },
                    {
                        setTopic: topic,
                    },
                ],
            },
        },
        {
            setTopic: topic,
        },
    ]
    claimerLocation = claimerLocation ?? sourceLocation
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                withdrawAsset: assets,
            },
            {
                payfees: {
                    asset: {
                        id: HERE_LOCATION,
                        fun: {
                            Fungible: localNativeFeeAmount,
                        },
                    },
                },
            },
            {
                setAppendix: [
                    {
                        setHints: {
                            hints: [{ assetClaimer: { location: sourceLocation } }],
                        },
                    },
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
                initiateTransfer: {
                    destination: parachainLocation(assetHubParaId),
                    remote_fees: {
                        teleport: {
                            definite: [
                                {
                                    id: HERE_LOCATION,
                                    fun: {
                                        Fungible:
                                            totalNativeFeeAmount -
                                            localNativeFeeAmount -
                                            remoteEtherFeeNativeAmount,
                                    },
                                },
                            ],
                        },
                    },
                    preserveOrigin: true,
                    assets: [
                        {
                            teleport: {
                                definite: [
                                    {
                                        id: HERE_LOCATION,
                                        fun: {
                                            Fungible: remoteEtherFeeNativeAmount,
                                        },
                                    },
                                ],
                            },
                        },
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
                    remoteXcm: remoteInstructionsOnAH,
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}
