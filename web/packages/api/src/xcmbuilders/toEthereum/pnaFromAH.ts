import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    accountToLocation,
    HERE_LOCATION,
    isRelaychainLocation,
    buildEthereumInstructions,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"
import { DeliveryFeeV2 } from "../../toEthereumSnowbridgeV2"

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
            reserveAssetDeposited: [
                {
                    id: asset.locationOnEthereum,
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
    fee: DeliveryFeeV2,
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
        if (!fee.feeLocation) {
            assets.push({
                id: bridgeLocation(ethChainId),
                fun: {
                    Fungible: remoteEtherFeeAmount,
                },
            })
        }
    } else {
        // native asset first
        if (tokenLocation.parents == 0) {
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
        } // Parachain assets or KSM assets
        else if (tokenLocation.parents == 1 || tokenLocation.parents == 2) {
            assets.push({
                id: DOT_LOCATION,
                fun: {
                    Fungible: totalDOTFeeAmount + remoteEtherFeeAmount,
                },
            })
            assets.push({
                id: tokenLocation,
                fun: {
                    Fungible: tokenAmount,
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
        }
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
