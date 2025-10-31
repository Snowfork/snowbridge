import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    erc20LocationReanchored,
    accountToLocation,
    HERE_LOCATION,
    isEthereumNative,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"

export function buildExportXcm(
    registry: Registry,
    ethChainId: number,
    asset: Asset,
    sender: string,
    beneficiary: string,
    topic: string,
    transferAmount: bigint,
    feeInEther: bigint,
) {
    let senderLocation = accountToLocation(sender)
    let beneficiaryLocation = accountToLocation(beneficiary)
    let exportXcm: any[] = [
        {
            withdrawAsset: [
                {
                    id: HERE_LOCATION,
                    fun: {
                        Fungible: feeInEther,
                    },
                },
            ],
        },
        {
            payFees: {
                asset: {
                    id: HERE_LOCATION,
                    fun: {
                        Fungible: feeInEther,
                    },
                },
            },
        },
        {
            withdrawAsset: [
                {
                    id: erc20LocationReanchored(asset.token),
                    fun: {
                        Fungible: transferAmount,
                    },
                },
            ],
        },
        {
            aliasOrigin: {
                parents: 0,
                interior: {
                    x1: [senderLocation],
                },
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
        {
            setTopic: topic,
        },
    ]

    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                unpaidExecution: {
                    weight_limit: "unlimited",
                    check_origin: null,
                },
            },
            {
                exportMessage: {
                    network: { Ethereum: { chain_id: ethChainId } },
                    destination: "Here",
                    xcm: exportXcm,
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}

export function buildTransferXcmFromAssetHub(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    totalDOTFeeAmount: bigint,
    remoteEtherFeeAmount: bigint,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = erc20Location(ethChainId, asset.token)
    let assets = []

    assets.push({
        id: DOT_LOCATION,
        fun: {
            Fungible: totalDOTFeeAmount,
        },
    })
    if (isEthereumNative(tokenLocation, ethChainId)) {
        assets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: tokenAmount + remoteEtherFeeAmount,
            },
        })
    } else {
        assets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: remoteEtherFeeAmount,
            },
        })
        assets.push({
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        })
    }

    let instructions: any[] = [
        {
            withdrawAsset: assets,
        },
        {
            payfees: {
                asset: {
                    id: DOT_LOCATION,
                    fun: {
                        Fungible: totalDOTFeeAmount,
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
                                allCounted: 2,
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
                                    allCounted: 2,
                                },
                            },
                            beneficiary: {
                                parents: 0,
                                interior: { x1: [beneficiaryLocation] },
                            },
                        },
                    },
                ],
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
