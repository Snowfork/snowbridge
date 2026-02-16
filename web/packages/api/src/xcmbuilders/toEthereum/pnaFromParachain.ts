import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    ethereumNetwork,
    parachainLocation,
    accountToLocation,
    buildAppendixInstructions,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toEthereum_v2"

export function buildResultXcmAssetHubPNATransferFromParachain(
    registry: Registry,
    ethChainId: number,
    assetLocationOnAH: any,
    assetLocationOnEthereum: any,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    transferAmount: bigint,
    totalFeeInDot: bigint,
    destinationFeeInDot: bigint,
) {
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                withdrawAsset: [
                    {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: totalFeeInDot,
                        },
                    },
                ],
            },
            {
                buyExecution: {
                    fees: {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: destinationFeeInDot,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                receiveTeleportedAsset: [
                    {
                        id: assetLocationOnAH,
                        fun: {
                            Fungible: transferAmount,
                        },
                    },
                ],
            },
            { clearOrigin: null },
            ...buildAssetHubXcmForPNAFromParachain(
                ethChainId,
                beneficiary,
                assetLocationOnAH,
                assetLocationOnEthereum,
                topic,
            ),
        ],
    })
}

function buildAssetHubXcmForPNAFromParachain(
    ethChainId: number,
    beneficiary: string,
    assetLocationOnAH: any,
    assetLocationOnEthereum: any,
    topic: string,
) {
    return [
        // Initiate the bridged transfer
        {
            depositReserveAsset: {
                assets: {
                    Wild: {
                        AllOf: { id: assetLocationOnAH, fun: "Fungible" },
                    },
                },
                dest: bridgeLocation(ethChainId),
                xcm: [
                    {
                        buyExecution: {
                            fees: {
                                id: assetLocationOnEthereum, // CAUTION: Must use reanchored locations.
                                fun: {
                                    Fungible: "1", // Offering 1 unit as fee, but it is returned to the beneficiary address.
                                },
                            },
                            weight_limit: "Unlimited",
                        },
                    },
                    {
                        depositAsset: {
                            assets: {
                                Wild: {
                                    AllCounted: 1,
                                },
                            },
                            beneficiary: {
                                parents: 0,
                                interior: { x1: [{ AccountKey20: { key: beneficiary } }] },
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
}

export function buildParachainPNAReceivedXcmOnDestination(
    registry: Registry,
    assetLocation: any,
    transferAmount: bigint,
    feeInDot: bigint,
    beneficiary: string,
    topic: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                reserveAssetDeposited: [
                    {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: feeInDot,
                        },
                    },
                ],
            },
            {
                buyExecution: {
                    fees: {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: feeInDot,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                receiveTeleportedAsset: [
                    {
                        id: assetLocation,
                        fun: {
                            Fungible: transferAmount,
                        },
                    },
                ],
            },
            { clearOrigin: null },
            {
                depositAsset: {
                    assets: {
                        wild: {
                            allCounted: 2,
                        },
                    },
                    beneficiary: {
                        parents: 0,
                        interior: { x1: [beneficiaryLocation] },
                    },
                },
            },
            { setTopic: topic },
        ],
    })
}

export function buildAssetHubPNATransferFromParachain(
    registry: Registry,
    ethChainId: number,
    beneficiary: string,
    assetLocationOnAH: any,
    assetLocationOnEthereum: any,
    topic: string,
) {
    return registry.createType("XcmVersionedXcm", {
        v5: buildAssetHubXcmForPNAFromParachain(
            ethChainId,
            beneficiary,
            assetLocationOnAH,
            assetLocationOnEthereum,
            topic,
        ),
    })
}

export function buildParachainPNAReceivedXcmOnAssetHub(
    registry: Registry,
    ethChainId: number,
    assetLocationOnAH: any,
    destinationParaId: number,
    transferAmount: bigint,
    totalFeeInDot: bigint,
    destinationFeeInDot: bigint,
    beneficiary: string,
    topic: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                receiveTeleportedAsset: [
                    {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: totalFeeInDot,
                        },
                    },
                ],
            },
            {
                buyExecution: {
                    fees: {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: totalFeeInDot,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                descendOrigin: { x1: [{ PalletInstance: 80 }] },
            },
            {
                universalOrigin: ethereumNetwork(ethChainId),
            },
            {
                withdrawAsset: [
                    {
                        id: assetLocationOnAH,
                        fun: {
                            Fungible: transferAmount,
                        },
                    },
                ],
            },
            { clearOrigin: null },
            {
                setAppendix: [
                    {
                        depositAsset: {
                            assets: {
                                wild: {
                                    allCounted: 2,
                                },
                            },
                            beneficiary: bridgeLocation(ethChainId),
                        },
                    },
                ],
            },
            {
                reserveAssetDeposited: [
                    {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: destinationFeeInDot,
                        },
                    },
                ],
            },
            {
                initiateTeleport: {
                    assets: {
                        definite: [
                            {
                                id: assetLocationOnAH,
                                fun: {
                                    Fungible: transferAmount,
                                },
                            },
                        ],
                    },
                    dest: { parents: 1, interior: { x1: [{ parachain: destinationParaId }] } },
                    xcm: [
                        {
                            buyExecution: {
                                fees: {
                                    id: DOT_LOCATION,
                                    fun: {
                                        Fungible: destinationFeeInDot,
                                    },
                                },
                                weightLimit: "Unlimited",
                            },
                        },
                        {
                            depositAsset: {
                                assets: {
                                    wild: {
                                        allCounted: 2,
                                    },
                                },
                                beneficiary: {
                                    parents: 0,
                                    interior: { x1: [beneficiaryLocation] },
                                },
                            },
                        },
                        { setTopic: topic },
                    ],
                },
            },
            { setTopic: topic },
        ],
    })
}

export function buildTransferXcmFromParachain(
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
        (fee.localExecutionFeeDOT ?? 0n) + (fee.localDeliveryFeeDOT ?? 0n) + fee.returnToSenderExecutionFeeDOT
    let totalDOTFeeAmount: bigint = fee.totalFeeInDot!
    let remoteEtherFeeAmount: bigint = fee.ethereumExecutionFee!

    let assets = []
    assets.push({
        id: tokenLocation,
        fun: {
            Fungible: tokenAmount,
        },
    })
    assets.push({
        id: DOT_LOCATION,
        fun: {
            Fungible: totalDOTFeeAmount,
        },
    })
    assets.push({
        id: bridgeLocation(ethChainId),
        fun: {
            Fungible: remoteEtherFeeAmount,
        },
    })

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
                                        Fungible: totalDOTFeeAmount - localDOTFeeAmount,
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
                                        id: bridgeLocation(ethChainId),
                                        fun: {
                                            Fungible: remoteEtherFeeAmount,
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
