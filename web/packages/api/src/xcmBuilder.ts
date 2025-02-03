import { Registry } from "@polkadot/types/types";
import { beneficiaryMultiAddress } from "./utils";

export const DOT_LOCATION = { parents: 1, interior: "Here" }

const ethereumNetwork = (ethChainId: number) => ({ GlobalConsensus: { Ethereum: { chain_id: ethChainId } } })

export function bridgeLocation(ethChainId: number) {
    return {
        parents: 2,
        interior: { x1: [ethereumNetwork(ethChainId)] },
    }
}

export function erc20Location(ethChainId: number, tokenAddress: string) {
    return {
        parents: 2,
        interior: {
            X2: [
                ethereumNetwork(ethChainId),
                { AccountKey20: { key: tokenAddress } },
            ],
        },
    }
}

export function erc20LocationReanchored(tokenAddress: string) {
    return {
        parents: 0,
        interior: { X1: [{ AccountKey20: { key: tokenAddress } }] },
    }
}

export function buildERC20DestinationXcm(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    transferAmount: bigint,
    feeInDot: bigint,
    beneficiary: string,
    topic: string
) {
    let { hexAddress, address: { kind } } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation;
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break;
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break;
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType('XcmVersionedXcm',
        {
            v4: [
                {
                    reserveAssetDeposited: [
                        {
                            id: DOT_LOCATION,
                            fun: {
                                Fungible: feeInDot,
                            },
                        },
                        {
                            id: erc20Location(ethChainId, tokenAddress),
                            fun: {
                                Fungible: transferAmount,
                            },
                        }
                    ]
                },
                { clearOrigin: null },
                {
                    buyExecution: {
                        fees: {
                            id: DOT_LOCATION,
                            fun: {
                                Fungible: feeInDot,
                            },
                        },
                        weightLimit: "Unlimited",
                    }
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
                    }
                },
                { setTopic: topic }
            ]
        })
}

export function buildERC20AssetHubDestination(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    transferAmount: bigint,
    feeInDot: bigint,
    beneficiary: string,
    topic: string
) {
    let { hexAddress, address: { kind } } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation;
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break;
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break;
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType('XcmVersionedXcm',
        {
            v4: [
                {
                    receiveTeleportedAsset: [
                        {
                            id: DOT_LOCATION,
                            fun: {
                                Fungible: feeInDot,
                            },
                        },
                    ]
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
                    }
                },
                {
                    descendOrigin: { x1: [{ PalletInstance: 80 }] }
                },
                {
                    universalOrigin: ethereumNetwork(ethChainId)
                },
                {
                    reserveAssetDeposited: [
                        {
                            id: erc20Location(ethChainId, tokenAddress),
                            fun: {
                                Fungible: transferAmount,
                            },
                        }
                    ]
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
                    }
                },
                { setTopic: topic }
            ]
        }
    )
}

export function buildERC20ParachainDestination(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    destinationParaId: number,
    transferAmount: bigint,
    totalFeeInDot: bigint,
    destinationFeeInDot: bigint,
    beneficiary: string,
    topic: string
) {
    let { hexAddress, address: { kind } } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation;
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break;
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break;
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType('XcmVersionedXcm',
        {
            v4: [
                {
                    receiveTeleportedAsset: [
                        {
                            id: DOT_LOCATION,
                            fun: {
                                Fungible: totalFeeInDot,
                            },
                        },
                    ]
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
                    }
                },
                {
                    descendOrigin: { x1: [{ PalletInstance: 80 }] }
                },
                {
                    universalOrigin: ethereumNetwork(ethChainId)
                },
                {
                    reserveAssetDeposited: [
                        {
                            id: erc20Location(ethChainId, tokenAddress),
                            fun: {
                                Fungible: transferAmount,
                            },
                        }
                    ]
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
                            }
                        },
                    ]
                },
                {
                    depositReserveAsset: {
                        assets: {
                            definite: [
                                {
                                    id: DOT_LOCATION,
                                    fun: {
                                        Fungible: destinationFeeInDot,
                                    },
                                },
                                {
                                    id: erc20Location(ethChainId, tokenAddress),
                                    fun: {
                                        Fungible: transferAmount,
                                    },
                                }
                            ]
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
                                }
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
                                }
                            },
                            { setTopic: topic }
                        ]
                    }
                },
                { setTopic: topic }
            ]
        }
    )
}

export function buildERC20TransferFromSource(ethChainId: number, sourceAccount: string, beneficiary: string, tokenAddress: string, topic: string) {
    return [
        // Error Handling, return everything to sender on Asset hub
        {
          setAppendix: [
            {
              depositAsset: {
                assets: {
                  Wild: "All",
                },
                beneficiary: {
                  parents: 0,
                  interior: {
                    x1: [{
                      AccountId32: { id: sourceAccount },
                    }],
                  },
                },
              },
            },
          ],
        },
        // Initiate the bridged transfer
        {
          initiateReserveWithdraw: {
            assets: {
              Wild: {
                AllOf: { id: erc20Location(ethChainId, tokenAddress), fun: "Fungible" },
              },
            },
            reserve: bridgeLocation(ethChainId),
            xcm: [
              {
                buyExecution: {
                  fees: {
                    id: erc20LocationReanchored(tokenAddress), // CAUTION: Must use reanchored locations.
                    fun: {
                      Fungible: "1", // Offering 1 unit as fee, but it is returned to the destination address.
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
                setTopic: topic
              },
            ],
          },
        },
        {
          setTopic: topic
        },
      ];
}

export function buildERC20AssetHubPassthrough() {
    throw Error()
}