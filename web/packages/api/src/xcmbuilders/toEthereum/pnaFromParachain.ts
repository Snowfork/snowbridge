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
import { DeliveryFeeV2 } from "../../toEthereumSnowbridgeV2"

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
    fee: DeliveryFeeV2,
    claimerLocation?: any,
    callHex?: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let sourceLocation = accountToLocation(sourceAccount)
    let tokenLocation = asset.location

    let localDOTFeeAmount: bigint = fee.localExecutionFeeDOT! + fee.localDeliveryFeeDOT!
    let totalDOTFeeAmount: bigint = fee.totalFeeInDot!
    let remoteEtherFeeAmount: bigint = fee.ethereumExecutionFee!
    let remoteEtherFeeInDOTAmount: bigint = fee.ethereumExecutionFeeInNative!

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
    if (!fee.feeLocation) {
        assets.push({
            id: bridgeLocation(ethChainId),
            fun: {
                Fungible: remoteEtherFeeAmount,
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
                                    remoteEtherFeeInDOTAmount > 0
                                        ? {
                                              id: DOT_LOCATION,
                                              fun: {
                                                  Fungible: remoteEtherFeeInDOTAmount,
                                              },
                                          }
                                        : {
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
