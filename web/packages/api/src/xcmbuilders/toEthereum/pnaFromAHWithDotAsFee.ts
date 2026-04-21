import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    accountToLocation,
    isRelaychainLocation,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { DOT_LOCATION } from "../../assets_v2"
import { Asset } from "@snowbridge/base-types"
import { DeliveryFee } from "../../toEthereum_v2"

export function buildTransferXcmFromAssetHubWithDOTAsFee(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    asset: Asset,
    tokenAmount: bigint,
    fee: DeliveryFee,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = asset.location

    let localDOTFeeAmount =
        fee.localExecutionFeeDOT! + fee.bridgeHubDeliveryFeeDOT + fee.snowbridgeDeliveryFeeDOT
    let totalDOTFeeAmount = fee.totalFeeInDot!
    let remoteEtherFeeAmount = fee.ethereumExecutionFee!

    let assets = []
    if (isRelaychainLocation(tokenLocation)) {
        assets.push({
            id: DOT_LOCATION,
            fun: {
                Fungible: totalDOTFeeAmount + tokenAmount,
            },
        })
    } else {
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
    }

    // Sort assets by parents because XCM requires it for binary search.
    assets.sort((a, b) => {
        return a.id.parents - b.id.parents
    })

    let remoteXcm = buildEthereumInstructions(beneficiaryLocation, topic, callHex)

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
                        reserveDeposit: {
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
