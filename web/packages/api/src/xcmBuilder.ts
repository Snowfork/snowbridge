import { Registry } from "@polkadot/types/types"
import { beneficiaryMultiAddress } from "./utils"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"

export const DOT_LOCATION = { parents: 1, interior: "Here" }

const ethereumNetwork = (ethChainId: number) => ({
    GlobalConsensus: { Ethereum: { chain_id: ethChainId } },
})

export function bridgeLocation(ethChainId: number) {
    return {
        parents: 2,
        interior: { x1: [ethereumNetwork(ethChainId)] },
    }
}

export function parahchainLocation(paraId: number) {
    return {
        parents: 1,
        interior: { x1: [{ parachain: paraId }] },
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

function buildAssetHubXcmFromParachain(
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    sourceParachainId: number,
    destinationFeeInDOT: bigint
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
                                        id: DOT_LOCATION,
                                        fun: {
                                            fungible: destinationFeeInDOT,
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

export function buildAssetHubERC20TransferFromParachain(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    tokenAddress: string,
    topic: string,
    sourceParachainId: number,
    returnToSenderFeeInDOT: bigint
) {
    return registry.createType("XcmVersionedXcm", {
        v4: buildAssetHubXcmFromParachain(
            ethChainId,
            sourceAccount,
            beneficiary,
            tokenAddress,
            topic,
            sourceParachainId,
            returnToSenderFeeInDOT
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
    totalFeeInDot: bigint,
    destinationFeeInDot: bigint,
    sourceParachainId: number,
    returnToSenderFeeInDOT: bigint
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
                            Fungible: destinationFeeInDot,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            ...buildAssetHubXcmFromParachain(
                ethChainId,
                sourceAccount,
                beneficiary,
                tokenAddress,
                topic,
                sourceParachainId,
                returnToSenderFeeInDOT
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
                sourceAccount,
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
    sourceAccount: string,
    beneficiary: string,
    assetLocationOnAH: any,
    assetLocationOnEthereum: any,
    topic: string
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
    sourceAccount: string,
    beneficiary: string,
    assetLocationOnAH: any,
    assetLocationOnEthereum: any,
    topic: string
) {
    return registry.createType("XcmVersionedXcm", {
        v4: buildAssetHubXcmForPNAFromParachain(
            ethChainId,
            sourceAccount,
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
