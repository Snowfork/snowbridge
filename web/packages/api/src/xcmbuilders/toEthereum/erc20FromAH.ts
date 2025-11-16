import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    erc20LocationReanchored,
    accountToLocation,
    HERE_LOCATION,
    buildEthereumInstructions,
    containsEther,
    splitEtherAsset,
} from "../../xcmBuilder"
import { ConcreteAsset } from "../../assets_v2"
import { DeliveryFeeV2 } from "../../toEthereumSnowbridgeV2"

export function buildExportXcm(
    registry: Registry,
    ethChainId: number,
    sender: string,
    beneficiary: string,
    topic: string,
    concreteAssets: ConcreteAsset[],
    deliveryFee: DeliveryFeeV2,
) {
    let senderLocation = accountToLocation(sender)
    let beneficiaryLocation = accountToLocation(beneficiary)
    let assets = []
    for (const asset of concreteAssets) {
        const tokenLocation = erc20LocationReanchored(asset.id.token)
        assets.push({
            id: tokenLocation,
            fun: {
                Fungible: asset.amount,
            },
        })
    }
    let exportXcm: any[] = [
        {
            withdrawAsset: [
                {
                    id: HERE_LOCATION,
                    fun: {
                        Fungible: deliveryFee.ethereumExecutionFee!,
                    },
                },
            ],
        },
        {
            payFees: {
                asset: {
                    id: HERE_LOCATION,
                    fun: {
                        Fungible: deliveryFee.ethereumExecutionFee!,
                    },
                },
            },
        },
        {
            withdrawAsset: assets,
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
    concreteAssets: ConcreteAsset[],
    fee: DeliveryFeeV2,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)

    let totalDOTFeeAmount = fee.totalFeeInDot
    let remoteEtherFeeAmount = fee.ethereumExecutionFee!

    let assets = [],
        reserveWithdrawAssets: any[] = []

    assets.push({
        id: DOT_LOCATION,
        fun: {
            Fungible: totalDOTFeeAmount,
        },
    })
    if (!containsEther(ethChainId, concreteAssets)) {
        if (!fee.feeLocation) {
            assets.push({
                id: bridgeLocation(ethChainId),
                fun: {
                    Fungible: remoteEtherFeeAmount,
                },
            })
        }
        for (const asset of concreteAssets) {
            const tokenLocation = erc20Location(ethChainId, asset.id.token)
            const tokenAmount = asset.amount
            assets.push({
                id: tokenLocation,
                fun: {
                    Fungible: tokenAmount,
                },
            })
        }
    } else {
        const { etherAsset, otherAssets } = splitEtherAsset(ethChainId, concreteAssets)
        assets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: etherAsset!.amount + remoteEtherFeeAmount,
            },
        })
        for (const asset of otherAssets) {
            const tokenLocation = erc20Location(ethChainId, asset.id.token)
            const tokenAmount = asset.amount
            assets.push({
                id: tokenLocation,
                fun: {
                    Fungible: tokenAmount,
                },
            })
        }
    }
    for (const asset of concreteAssets) {
        const tokenLocation = erc20Location(ethChainId, asset.id.token)
        const tokenAmount = asset.amount
        reserveWithdrawAssets.push({
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        })
    }

    let remoteXcm = buildEthereumInstructions(beneficiaryLocation, topic, callHex)

    let exchangeInstruction = fee.feeLocation
        ? {
              exchangeAsset: {
                  give: {
                      Wild: {
                          AllOf: {
                              id: fee.feeLocation,
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
          }
        : undefined

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
        ...(exchangeInstruction ? [exchangeInstruction] : []),
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
        v5: instructions,
    })
}
