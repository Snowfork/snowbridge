import { Registry } from "@polkadot/types/types"
import { beneficiaryMultiAddress } from "./utils"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"

export const HERE_LOCATION = { parents: 0, interior: "Here" }
export const DOT_LOCATION = { parents: 1, interior: "Here" }
export const NATIVE_TOKEN_LOCATION = { parents: 1, interior: "Here" }
export const polkadotNetwork = {
    GlobalConsensus: { Polkadot: { network: null } },
}
export const kusamaNetwork = {
    GlobalConsensus: { Kusama: { network: null } },
}
export const dotLocationOnKusamaAssetHub = {
    parents: 2,
    interior: { x1: [{ GlobalConsensus: { Polkadot: null } }] },
}
export const ksmLocationOnPolkadotAssetHub = {
    parents: 2,
    interior: { x1: [{ GlobalConsensus: { Kusama: null } }] },
}

export const ethereumNetwork = (ethChainId: number) => ({
    GlobalConsensus: { Ethereum: { chain_id: ethChainId } },
})

export function bridgeLocation(ethChainId: number) {
    return {
        parents: 2,
        interior: { x1: [ethereumNetwork(ethChainId)] },
    }
}

export function parachainLocation(paraId: number) {
    return {
        parents: 1,
        interior: { x1: [{ parachain: paraId }] },
    }
}

export function accountId32Location(hexAddress: string) {
    return {
        parents: 0,
        interior: { x1: [{ accountId32: { id: hexAddress } }] },
    }
}

export function kusamaAssetHubLocation(parachainId: number) {
    return {
        parents: 2,
        interior: { x2: [{ GlobalConsensus: { Kusama: null } }, { parachain: parachainId }] },
    }
}

export function polkadotAssetHubLocation(parachainId: number) {
    return {
        parents: 2,
        interior: { x2: [{ GlobalConsensus: { Polkadot: null } }, { parachain: parachainId }] },
    }
}

export function erc20Location(ethChainId: number, tokenAddress: string) {
    if (tokenAddress === ETHER_TOKEN_ADDRESS) {
        return bridgeLocation(ethChainId)
    }
    return {
        parents: 2,
        interior: {
            X2: [ethereumNetwork(ethChainId), { AccountKey20: { key: tokenAddress } }],
        },
    }
}

export function erc20LocationReanchored(tokenAddress: string) {
    if (tokenAddress === ETHER_TOKEN_ADDRESS) {
        return {
            parents: 0,
            interior: { here: null },
        }
    }
    return {
        parents: 0,
        interior: { X1: [{ AccountKey20: { key: tokenAddress } }] },
    }
}

export function convertToXcmV3X1(location: any) {
    if (location.interior.x1) {
        const convertedLocation = JSON.parse(JSON.stringify(location))
        convertedLocation.interior.x1 = convertedLocation.interior.x1[0]
        return convertedLocation
    }
    return location
}

export function getTokenFromLocation(location: any, chainId: number) {
    if (location.parents === 2) {
        // New XCM multi-location format. x1 is an array.
        if (
            location.interior.x1 &&
            location.interior.x1[0]?.globalConsensus?.ethereum?.chainId === chainId
        ) {
            return ETHER_TOKEN_ADDRESS
        }
        // Old XCM multi-location format. x1 is not an array.
        if (
            location.interior.x1 &&
            location.interior.x1.globalConsensus?.ethereum?.chainId === chainId
        ) {
            return ETHER_TOKEN_ADDRESS
        }
        if (
            location.interior.x2 &&
            location.interior.x2[0]?.globalConsensus?.ethereum?.chainId === chainId &&
            location.interior.x2[1].accountKey20
        ) {
            const token = String(location.interior.x2[1].accountKey20.key.toLowerCase())
            if (token !== ETHER_TOKEN_ADDRESS) {
                return token
            }
        }
    }
    return undefined
}

export function buildParachainERC20ReceivedXcmOnDestination(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    transferAmount: bigint,
    feeInDot: bigint,
    beneficiary: string,
    topic: string
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
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
                    },
                ],
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
    })
}

export function buildAssetHubERC20ReceivedXcm(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    transferAmount: bigint,
    feeInDot: bigint,
    beneficiary: string,
    topic: string
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                receiveTeleportedAsset: [
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
                descendOrigin: { x1: [{ PalletInstance: 80 }] },
            },
            {
                universalOrigin: ethereumNetwork(ethChainId),
            },
            {
                reserveAssetDeposited: [
                    {
                        id: erc20Location(ethChainId, tokenAddress),
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

export function buildParachainERC20ReceivedXcmOnAssetHub(
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
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
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
                reserveAssetDeposited: [
                    {
                        id: erc20Location(ethChainId, tokenAddress),
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
                depositReserveAsset: {
                    // Should use `AllCounted` here. Reference:
                    // https://github.com/paritytech/polkadot-sdk/blob/f5de39196e8c30de4bc47a2d46b1a0fe1e9aaee0/bridges/snowbridge/primitives/inbound-queue/src/v1.rs#L357-L359
                    assets: {
                        wild: {
                            AllCounted: 2,
                        },
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

function buildAssetHubXcmFromParachainKusama(beneficiary: string, topic: string) {
    return [
        {
            depositAsset: {
                assets: {
                    Wild: {
                        AllCounted: 2,
                    },
                },
                beneficiary: {
                    parents: 0,
                    interior: { x1: [{ AccountId32: { id: beneficiary } }] },
                },
            },
        },
        {
            setTopic: topic,
        },
    ]
}

function buildAssetHubXcmFromParachain(
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    sourceParachainId: number,
    destinationFee: bigint,
    feeAssetId: any
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(sourceAccount)
    let sourceAccountLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            sourceAccountLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            sourceAccountLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse source address ${sourceAccount}`)
    }
    return [
        // Error Handling, return everything to sender on source parachain
        {
            setAppendix: [
                {
                    depositReserveAsset: {
                        assets: {
                            wild: "All",
                        },
                        dest: { parents: 1, interior: { x1: [{ parachain: sourceParachainId }] } },
                        xcm: [
                            {
                                buyExecution: {
                                    fees: {
                                        id: feeAssetId,
                                        fun: {
                                            fungible: destinationFee,
                                        },
                                    },
                                    weightLimit: "Unlimited",
                                },
                            },
                            {
                                depositAsset: {
                                    assets: {
                                        wild: "All",
                                    },
                                    beneficiary: {
                                        parents: 0,
                                        interior: { x1: [sourceAccountLocation] },
                                    },
                                },
                            },
                            { setTopic: topic },
                        ],
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

export function buildAssetHubERC20TransferToKusama(
    registry: Registry,
    beneficiary: string,
    topic: string
) {
    return registry.createType("XcmVersionedXcm", {
        v4: buildAssetHubXcmFromParachainKusama(beneficiary, topic),
    })
}

export function buildAssetHubERC20TransferFromParachain(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    sourceParachainId: number,
    returnToSenderFee: bigint,
    feeAssetId: any
) {
    return registry.createType("XcmVersionedXcm", {
        v4: buildAssetHubXcmFromParachain(
            ethChainId,
            sourceAccount,
            beneficiary,
            tokenAddress,
            topic,
            sourceParachainId,
            returnToSenderFee,
            feeAssetId
        ),
    })
}

export function buildResultXcmAssetHubERC20TransferFromParachain(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    transferAmount: bigint,
    totalFee: bigint,
    destinationFee: bigint,
    sourceParachainId: number,
    returnToSenderFee: bigint,
    feeAssetId: any,
    feeAssetIdReanchored: any,
    teleportFee: boolean
) {
    return registry.createType("XcmVersionedXcm", {
        v4: [
            ...(teleportFee
                ? // Teleport Fee
                  [
                      {
                          receiveTeleportedAsset: [
                              {
                                  id: feeAssetIdReanchored,
                                  fun: {
                                      Fungible: totalFee,
                                  },
                              },
                          ],
                      },
                      {
                          buyExecution: {
                              fees: {
                                  id: feeAssetIdReanchored,
                                  fun: {
                                      Fungible: destinationFee,
                                  },
                              },
                              weightLimit: "Unlimited",
                          },
                      },
                      {
                          withdrawAsset: [
                              {
                                  id: erc20Location(ethChainId, tokenAddress),
                                  fun: {
                                      Fungible: transferAmount,
                                  },
                              },
                          ],
                      },
                  ]
                : // Reserve Transfer Fee
                  [
                      {
                          withdrawAsset: [
                              {
                                  id: feeAssetIdReanchored,
                                  fun: {
                                      Fungible: totalFee,
                                  },
                              },
                              {
                                  id: erc20Location(ethChainId, tokenAddress),
                                  fun: {
                                      Fungible: transferAmount,
                                  },
                              },
                          ],
                      },
                      {
                          buyExecution: {
                              fees: {
                                  id: feeAssetIdReanchored,
                                  fun: {
                                      Fungible: destinationFee,
                                  },
                              },
                              weightLimit: "Unlimited",
                          },
                      },
                  ]),
            { clearOrigin: null },
            ...buildAssetHubXcmFromParachain(
                ethChainId,
                sourceAccount,
                beneficiary,
                tokenAddress,
                topic,
                sourceParachainId,
                returnToSenderFee,
                feeAssetId
            ),
        ],
    })
}

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
    destinationFeeInDot: bigint
) {
    return registry.createType("XcmVersionedXcm", {
        v4: [
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
                topic
            ),
        ],
    })
}

function buildAssetHubXcmForPNAFromParachain(
    ethChainId: number,
    beneficiary: string,
    assetLocationOnAH: any,
    assetLocationOnEthereum: any,
    topic: string
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
    topic: string
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
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
    topic: string
) {
    return registry.createType("XcmVersionedXcm", {
        v4: buildAssetHubXcmForPNAFromParachain(
            ethChainId,
            beneficiary,
            assetLocationOnAH,
            assetLocationOnEthereum,
            topic
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
    topic: string
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
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

export function buildAssetHubPNAReceivedXcm(
    registry: Registry,
    ethChainId: number,
    assetLocation: any,
    transferAmount: bigint,
    feeInDot: bigint,
    beneficiary: string,
    topic: string
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                receiveTeleportedAsset: [
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
                descendOrigin: { x1: [{ PalletInstance: 80 }] },
            },
            {
                universalOrigin: ethereumNetwork(ethChainId),
            },
            {
                withdrawAsset: [
                    {
                        id: assetLocation,
                        fun: {
                            Fungible: transferAmount,
                        },
                    },
                ],
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
    })
}

export function buildExportXcmForERC20(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    beneficiary: string,
    topic: string,
    transferAmount: bigint,
    totalFeeInDot: bigint,
    assetHubParaId: number
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
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
                            Fungible: totalFeeInDot,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                setAppendix: [
                    {
                        depositAsset: {
                            assets: {
                                wild: {
                                    allCounted: 1,
                                },
                            },
                            beneficiary: parachainLocation(assetHubParaId),
                        },
                    },
                ],
            },
            {
                exportMessage: {
                    network: { Ethereum: { chain_id: ethChainId } },
                    destination: "Here",
                    xcm: [
                        {
                            withdrawAsset: [
                                {
                                    id: erc20LocationReanchored(tokenAddress),
                                    fun: {
                                        Fungible: transferAmount,
                                    },
                                },
                            ],
                        },
                        { clearOrigin: null },
                        {
                            buyExecution: {
                                fees: {
                                    id: erc20LocationReanchored(tokenAddress),
                                    fun: {
                                        Fungible: "1",
                                    },
                                },
                                weight_limit: "Unlimited",
                            },
                        },
                        {
                            depositAsset: {
                                assets: {
                                    wild: {
                                        allCounted: 1,
                                    },
                                },
                                beneficiary: parachainLocation(1000),
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

export function buildExportXcmForPNA(
    registry: Registry,
    ethChainId: number,
    assetLocationOnEthereum: any,
    beneficiary: string,
    topic: string,
    transferAmount: bigint,
    totalFeeInDot: bigint,
    assetHubParaId: number
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
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
                            Fungible: totalFeeInDot,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                setAppendix: [
                    {
                        depositAsset: {
                            assets: {
                                wild: {
                                    allCounted: 1,
                                },
                            },
                            beneficiary: parachainLocation(assetHubParaId),
                        },
                    },
                ],
            },
            {
                exportMessage: {
                    network: { Ethereum: { chain_id: ethChainId } },
                    destination: "Here",
                    xcm: [
                        {
                            reserveAssetDeposited: [
                                {
                                    id: assetLocationOnEthereum,
                                    fun: {
                                        Fungible: transferAmount,
                                    },
                                },
                            ],
                        },
                        { clearOrigin: null },
                        {
                            buyExecution: {
                                fees: {
                                    id: assetLocationOnEthereum,
                                    fun: {
                                        Fungible: "1",
                                    },
                                },
                                weight_limit: "Unlimited",
                            },
                        },
                        {
                            depositAsset: {
                                assets: {
                                    wild: {
                                        allCounted: 1,
                                    },
                                },
                                beneficiary: parachainLocation(1000),
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

export function isKSMOnOtherConsensusSystem(location: any) {
    return matchesConsensusSystem(location, "Kusama")
}

export function isDOTOnOtherConsensusSystem(location: any): boolean {
    return matchesConsensusSystem(location, "Polkadot")
}

export function matchesConsensusSystem(location: any, expectedSystem: string): boolean {
    if (location.parents !== 2 || !location.interior) return false

    const kind = Object.keys(location.interior).find((k) => k.toLowerCase() === "x1")
    if (!kind) return false

    const values = location.interior[kind]
    if (!Array.isArray(values) || values.length === 0) return false

    const consensus = values[0]
    const consensusKey = Object.keys(consensus || {}).find(
        (k) => k.toLowerCase() === "globalconsensus"
    )
    if (!consensusKey) return false

    const consensusValue = consensus[consensusKey]
    return (
        typeof consensusValue === "object" &&
        Object.keys(consensusValue).some((k) => k.toLowerCase() === expectedSystem.toLowerCase())
    )
}

export function isEthereumAsset(location: any): boolean {
    if (location.parents !== 2 || !location.interior) return false

    const interior = location.interior

    const kind = Object.keys(interior).find(
        (k) => k.toLowerCase() === "x1" || k.toLowerCase() === "x2"
    )

    if (!kind) return false

    const values = interior[kind]
    if (!Array.isArray(values) || values.length === 0) return false

    const consensus = values[0]

    const consensusKey = Object.keys(consensus || {}).find(
        (k) => k.toLowerCase() === "globalconsensus"
    )

    if (!consensusKey) return false

    const consensusValue = consensus[consensusKey]

    return (
        typeof consensusValue === "object" &&
        Object.keys(consensusValue).some((k) => k.toLowerCase() === "ethereum")
    )
}

export function isRelaychainLocation(location: any) {
    return location.parents == DOT_LOCATION.parents && location.interior == DOT_LOCATION.interior
}

export function isParachainNative(location: any, parachainId: number) {
    return JSON.stringify(location) == JSON.stringify(parachainLocation(parachainId))
}

export function isEthereumNative(location: any, ethChainId: number) {
    return JSON.stringify(location) == JSON.stringify(bridgeLocation(ethChainId))
}

export const accountToLocation = (account: string) => {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(account)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse beneficiary address ${account}`)
    }
    return beneficiaryLocation
}

export const WESTEND_GENESIS = "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e"
export const ROCOCO_GENESIS = "0x6408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e"
export const PASEO_GENESIS = "0x77afd6190f1554ad45fd0d31aee62aacc33c6db0ea801129acb813f913e0764f"

export const accountToLocationWithNetwork = (account: string, envName: string) => {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(account)
    let beneficiaryLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            switch (envName) {
                case "polkadot_mainnet": {
                    beneficiaryLocation = {
                        accountId32: { id: hexAddress, network: { Polkadot: { network: null } } },
                    }
                    break
                }
                case "paseo_sepolia": {
                    beneficiaryLocation = {
                        accountId32: { id: hexAddress, network: { byGenesis: PASEO_GENESIS } },
                    }
                    break
                }
                case "westend_sepolia": {
                    beneficiaryLocation = {
                        accountId32: { id: hexAddress, network: { byGenesis: WESTEND_GENESIS } },
                    }
                    break
                }
                case "local_e2e": {
                    beneficiaryLocation = {
                        accountId32: { id: hexAddress, network: { byGenesis: WESTEND_GENESIS } },
                    }
                    break
                }
            }
            break
        case 2:
            // 20 byte addresses
            switch (envName) {
                case "polkadot_mainnet": {
                    beneficiaryLocation = {
                        accountKey20: { key: hexAddress, network: { Polkadot: { network: null } } },
                    }
                    break
                }
                case "paseo_sepolia": {
                    beneficiaryLocation = {
                        accountKey20: { key: hexAddress, network: { Polkadot: { network: null } } },
                    }
                    break
                }
                case "westend_sepolia": {
                    beneficiaryLocation = {
                        accountKey20: { key: hexAddress, network: { Polkadot: { network: null } } },
                    }
                    break
                }
            }
            break
        default:
            throw Error(`Could not parse beneficiary address ${account}`)
    }
    return beneficiaryLocation
}

export function buildAssetHubERC20TransferFromParachainWithNativeFee(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    sourceParachainId: number,
    amount: bigint,
    returnToSenderFeeInDot: bigint,
    feeAssetId: any
) {
    return registry.createType("XcmVersionedXcm", {
        v4: buildAssetHubXcmFromParachainWithNativeAssetAsFee(
            ethChainId,
            sourceAccount,
            beneficiary,
            tokenAddress,
            topic,
            sourceParachainId,
            amount,
            returnToSenderFeeInDot,
            feeAssetId,
        ),
    })
}

function buildAssetHubXcmFromParachainWithNativeAssetAsFee(
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    sourceParachainId: number,
    amount: bigint,
    destinationFeeInDot: bigint,
    feeAssetId: any
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(sourceAccount)
    let sourceAccountLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            sourceAccountLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            sourceAccountLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse source address ${sourceAccount}`)
    }
    let appendixInstructions = [
        // Exchange some DOT to pay the fee on the source parachain
        {
            exchangeAsset: {
                give: {
                    Wild: {
                        AllOf: {
                            id: feeAssetId,
                            fun: "Fungible",
                        },
                    },
                },
                want: [
                    {
                        id: DOT_LOCATION,
                        fun: {
                            Fungible: destinationFeeInDot,
                        },
                    },
                ],
                maximal: false,
            },
        },
        // DepositReserveAsset for both DOT and the ERC-20 asset
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
                            fun: { Fungible: amount },
                        },
                    ],
                },
                dest: { parents: 1, interior: { x1: [{ parachain: sourceParachainId }] } },
                xcm: [
                    {
                        buyExecution: {
                            fees: {
                                id: DOT_LOCATION,
                                fun: {
                                    fungible: destinationFeeInDot,
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
                                interior: { x1: [sourceAccountLocation] },
                            },
                        },
                    },
                    { setTopic: topic },
                ],
            },
        },
    ]
    return [
        // Error Handling, return everything to sender on source parachain
        {
            setAppendix: appendixInstructions,
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

export function buildERC20ToAssetHubFromParachain(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    transferAmount: bigint,
    totalFee: bigint,
    destinationFee: bigint,
    feeAssetIdReanchored: any
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryAccountLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryAccountLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryAccountLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse source address ${sourceAccount}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                withdrawAsset: [
                    {
                        id: feeAssetIdReanchored,
                        fun: {
                            Fungible: totalFee,
                        },
                    },
                    {
                        id: erc20Location(ethChainId, tokenAddress),
                        fun: {
                            Fungible: transferAmount,
                        },
                    },
                ],
            },
            { clearOrigin: null },
            {
                buyExecution: {
                    fees: {
                        id: feeAssetIdReanchored,
                        fun: {
                            Fungible: destinationFee,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                depositAsset: {
                    assets: {
                        Wild: {
                            AllCounted: 2,
                        },
                    },
                    beneficiary: {
                        parents: 0,
                        interior: { x1: [beneficiaryAccountLocation] },
                    },
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}
export function buildDepositAllAssetsWithTopic(
    registry: Registry,
    beneficiary: string,
    topic: string
) {
    let {
        hexAddress,
        address: { kind },
    } = beneficiaryMultiAddress(beneficiary)
    let beneficiaryAccountLocation
    switch (kind) {
        case 1:
            // 32 byte addresses
            beneficiaryAccountLocation = { accountId32: { id: hexAddress } }
            break
        case 2:
            // 20 byte addresses
            beneficiaryAccountLocation = { accountKey20: { key: hexAddress } }
            break
        default:
            throw Error(`Could not parse source address ${beneficiary}`)
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                depositAsset: {
                    assets: {
                        Wild: {
                            AllCounted: 2,
                        },
                    },
                    beneficiary: {
                        parents: 0,
                        interior: { x1: [beneficiaryAccountLocation] },
                    },
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}

export function buildAppendixInstructions(
    envName: string,
    sourceParachainId: number,
    sourceAccount: string,
    claimerLocation?: any
) {
    let sourceLocation = accountToLocationWithNetwork(sourceAccount, envName)
    let appendixInstructions: any[] = []
    if (claimerLocation) {
        appendixInstructions.push({
            setHints: {
                hints: [{ assetClaimer: { location: claimerLocation } }],
            },
        })
    }
    appendixInstructions.push({
        refundSurplus: null,
    })
    appendixInstructions.push({
        depositAsset: {
            assets: {
                wild: {
                    allCounted: 3,
                },
            },
            beneficiary: claimerLocation ?? {
                parents: 1,
                interior: {
                    x2: [{ parachain: sourceParachainId }, sourceLocation],
                },
            },
        },
    })
    return appendixInstructions
}
