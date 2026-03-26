import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    parachainLocation,
    accountToLocation,
    HERE_LOCATION,
    buildAppendixInstructions,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toEthereum_v2"

export function buildTransferXcmFromParachainWithNativeAssetFee(
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
    let tokenLocation = erc20Location(ethChainId, asset.token)

    let localNativeFeeAmount =
        fee.localExecutionFeeInNative! +
        fee.localDeliveryFeeInNative! +
        fee.returnToSenderExecutionFeeNative!
    let totalNativeFeeAmount = fee.totalFeeInNative!
    let remoteEtherFeeAmount = fee.ethereumExecutionFee!
    let remoteEtherFeeNativeAmount = fee.ethereumExecutionFeeInNative!

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
