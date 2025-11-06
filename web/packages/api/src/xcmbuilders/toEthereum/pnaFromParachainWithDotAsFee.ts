import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    parachainLocation,
    accountToLocation,
    buildAppendixInstructions,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toEthereum_v2"

export function buildTransferXcmFromParachainWithDOTAsFee(
    registry: Registry,
    envName: string,
    ethChainId: number,
    assetHubParaId: number,
    sourceParachainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    fee: DeliveryFee,
    claimerLocation?: any,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = asset.location

    let localDOTFeeAmount: bigint =
        fee.localExecutionFeeDOT! + fee.localDeliveryFeeDOT! + fee.returnToSenderExecutionFeeDOT
    let totalDOTFeeAmount: bigint = fee.totalFeeInDot
    let remoteEtherFeeAmount: bigint = fee.ethereumExecutionFee!
    let remoteEtherFeeInDOTAmount: bigint = fee.ethereumExecutionFeeInNative!

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

    let appendixInstructions = buildAppendixInstructions(
        envName,
        sourceParachainId,
        sourceAccount,
        claimerLocation,
    )

    let remoteXcm = buildEthereumInstructions(beneficiaryLocation, topic, callHex)

    let remoteInstructionsOnAH: any[] = [
        {
            setAppendix: appendixInstructions,
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
                remoteXcm: remoteXcm,
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
