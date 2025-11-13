import { Registry } from "@polkadot/types/types"
import { beneficiaryMultiAddress } from "../../utils"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    erc20LocationReanchored,
    parachainLocation,
    accountToLocation,
    isEthereumNative,
    buildAppendixInstructions,
    buildEthereumInstructions,
    containsEther,
    splitEtherAsset,
} from "../../xcmBuilder"
import { ConcreteAsset } from "../../assets_v2"
import { DeliveryFeeV2 } from "../../toEthereumSnowbridgeV2"

export function buildResultXcmAssetHubERC20TransferFromParachain(
    registry: Registry,
    ethChainId: number,
    sourceParachainId: number,
    sourceAccount: string,
    beneficiary: string,
    topic: string,
    concreteAssets: ConcreteAsset[],
    deliveryFee: DeliveryFeeV2,
) {
    let assets: any[] = [
        {
            id: DOT_LOCATION,
            fun: {
                Fungible: deliveryFee.totalFeeInDot!,
            },
        },
    ]
    for (const asset of concreteAssets) {
        assets.push({
            id: erc20Location(ethChainId, asset.id.token),
            fun: {
                Fungible: asset.amount,
            },
        })
    }
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                withdrawAsset: assets,
            },
            { clearOrigin: null },
            {
                buyExecution: {
                    fees: {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: deliveryFee.assetHubExecutionFeeDOT!,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                initiateReserveWithdraw: {
                    assets: {
                        Wild: {
                            AllOf: { id: bridgeLocation(ethChainId), fun: "Fungible" },
                        },
                    },
                    reserve: bridgeLocation(ethChainId),
                    xcm: [
                        {
                            buyExecution: {
                                fees: {
                                    id: bridgeLocation(ethChainId), // CAUTION: Must use reanchored locations.
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
    concreteAssets: ConcreteAsset[],
    fee: DeliveryFeeV2,
    claimerLocation?: any,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)

    let localDOTFeeAmount: bigint =
        fee.localExecutionFeeDOT! + fee.localDeliveryFeeDOT! + fee.returnToSenderExecutionFeeDOT
    let totalDOTFeeAmount: bigint = fee.totalFeeInDot!
    let remoteEtherFeeAmount: bigint = fee.ethereumExecutionFee!

    let assets = []
    assets.push({
        id: DOT_LOCATION,
        fun: {
            Fungible: totalDOTFeeAmount,
        },
    })
    if (!containsEther(ethChainId, concreteAssets)) {
        assets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: remoteEtherFeeAmount,
            },
        })
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

    let assetInstructions = []
    for (const asset of concreteAssets) {
        const tokenLocation = erc20Location(ethChainId, asset.id.token)
        const tokenAmount = asset.amount
        assetInstructions.push({
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
        })
    }

    let appendixInstructions = buildAppendixInstructions(
        envName,
        sourceParachainId,
        sourceAccount,
        claimerLocation,
    )

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

    let remoteInstructionsOnAH: any[] = [
        {
            setAppendix: appendixInstructions,
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
                assets: assetInstructions,
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
                        reserveWithdraw: {
                            definite: [
                                {
                                    id: DOT_LOCATION,
                                    fun: {
                                        Fungible:
                                            totalDOTFeeAmount -
                                            localDOTFeeAmount -
                                            remoteEtherFeeAmount,
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
                        ...assetInstructions,
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
