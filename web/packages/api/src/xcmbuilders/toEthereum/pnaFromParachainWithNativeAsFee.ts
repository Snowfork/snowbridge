import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    parachainLocation,
    accountToLocation,
    HERE_LOCATION,
    buildAppendixInstructions,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { DOT_LOCATION } from "../../assets_v2"
import { Asset } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toEthereum_v2"
import { findInBreakdownOrZero, findTotal } from "../../fees"

export function buildTransferXcmFromParachainWithNativeAssetFee(
    registry: Registry,
    envName: string,
    ethChainId: number,
    assetHubParaId: number,
    sourceParachainId: number,
    nativeSymbol: string,
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

    let localNativeFeeAmount =
        findInBreakdownOrZero(fee.breakdown, "localExecution", nativeSymbol) +
        findInBreakdownOrZero(fee.breakdown, "localDelivery", nativeSymbol) +
        findInBreakdownOrZero(fee.breakdown, "returnToSenderExecution", nativeSymbol)
    let totalNativeFeeAmount = findTotal(fee, nativeSymbol)
    let remoteEtherFeeAmount = findInBreakdownOrZero(fee.breakdown, "ethereumExecution", "ETH")
    let remoteEtherFeeNativeAmount = findInBreakdownOrZero(
        fee.breakdown,
        "ethereumExecution",
        nativeSymbol,
    )

    let assets = []
    if (JSON.stringify(HERE_LOCATION) == JSON.stringify(tokenLocation)) {
        assets.push({
            id: HERE_LOCATION,
            fun: {
                Fungible: totalNativeFeeAmount + tokenAmount,
            },
        })
    } else {
        assets.push({
            id: HERE_LOCATION,
            fun: {
                Fungible: totalNativeFeeAmount,
            },
        })
        assets.push({
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
