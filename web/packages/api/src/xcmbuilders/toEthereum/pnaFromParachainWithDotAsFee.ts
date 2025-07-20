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

export function buildTransferXcmFromParachainWithDOTAsFee(
    registry: Registry,
    ethChainId: number,
    assetHubParaId: number,
    sourceParachainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    localDOTFeeAmount: bigint,
    totalDOTFeeAmount: bigint,
    remoteEtherFeeAmount: bigint,
    remoteEtherFeeInDOTAmount: bigint
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = asset.location || erc20Location(ethChainId, asset.token)
    let assets = [
        {
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        },
        {
            id: DOT_LOCATION,
            fun: {
                Fungible: totalDOTFeeAmount,
            },
        },
    ]
    let remoteInstructionsOnAH: any[] = [
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
                        reserveDeposit: {
                            definite: [
                                {
                                    id: asset.locationOnAH,
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
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                withdrawAsset: assets,
            },
            {
                payfees: {
                    asset: {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: localDOTFeeAmount,
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
                initiateTransfer: {
                    destination: parachainLocation(assetHubParaId),
                    remote_fees: {
                        reserveWithdraw: {
                            definite: [
                                {
                                    id: DOT_LOCATION,
                                    fun: {
                                        Fungible:
                                            totalDOTFeeAmount -
                                            localDOTFeeAmount -
                                            remoteEtherFeeInDOTAmount,
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
                                        id: DOT_LOCATION,
                                        fun: {
                                            Fungible: remoteEtherFeeInDOTAmount,
                                        },
                                    },
                                ],
                            },
                        },
                        {
                            teleport: {
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
