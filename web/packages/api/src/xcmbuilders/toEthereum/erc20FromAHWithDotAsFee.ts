import { Registry } from "@polkadot/types/types"
import { bridgeLocation, DOT_LOCATION, erc20Location, accountToLocation } from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"

export function buildTransferXcmFromAssetHubWithDOTAsFee(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    localDOTFeeAmount: bigint,
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
    assets.push({
        id: tokenLocation,
        fun: {
            Fungible: tokenAmount,
        },
    })
    let instructions: any[] = [
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
