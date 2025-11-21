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
    splitEtherAsset,
} from "../../xcmBuilder"
import { DeliveryFeeV2 } from "../../toEthereumSnowbridgeV2"
import { ConcreteAsset } from "../../assets_v2"

export function buildTransferXcmFromParachainWithNativeAssetFee(
    registry: Registry,
    envName: string,
    ethChainId: number,
    assetHubParaId: number,
    sourceParachainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    concreteAssets: ConcreteAsset[],
    fee: DeliveryFeeV2,
    claimerLocation?: any,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)

    let localNativeFeeAmount = fee.localExecutionFeeInNative! + fee.localDeliveryFeeInNative!
    let totalNativeFeeAmount = fee.totalFeeInNative!
    let remoteEtherFeeAmount = fee.ethereumExecutionFee!
    let remoteEtherFeeNativeAmount = fee.ethereumExecutionFeeInNative!

    let assets = [],
        reserveWithdrawAssets = []
    assets.push({
        id: HERE_LOCATION,
        fun: {
            Fungible: totalNativeFeeAmount,
        },
    })
    const { etherAsset, otherAssets } = splitEtherAsset(ethChainId, concreteAssets)
    if (etherAsset) {
        assets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: etherAsset.amount,
            },
        })
        reserveWithdrawAssets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: etherAsset.amount,
            },
        })
    }
    for (const asset of otherAssets) {
        const tokenLocation = erc20Location(ethChainId, asset.id.token)
        const tokenAmount = asset.amount
        assets.push({
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        })
        reserveWithdrawAssets.push({
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        })
    }

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
                            definite: reserveWithdrawAssets,
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
                                    allCounted: 8,
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
                                definite: reserveWithdrawAssets,
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
