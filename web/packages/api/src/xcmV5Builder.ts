import { Registry } from "@polkadot/types/types"
import { beneficiaryMultiAddress, parseLocation } from "./utils"
import { ETHER_TOKEN_ADDRESS } from "./assets_v2"
import { Asset } from "./assets_v3"

export const HERE_LOCATION = { parents: 0, interior: "Here" }
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

export function parachainLocation(paraId: number) {
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
        v5: [
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
        v5: [
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
        v5: [
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
        v5: buildAssetHubXcmFromParachain(
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
    feeAssetIdReanchored: any
) {
    return registry.createType("XcmVersionedXcm", {
        v5: [
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
        v5: [
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
        v5: buildAssetHubXcmForPNAFromParachain(
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
        v5: [
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
        v5: [
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

export function buildTransferXcmFromAssetHub(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    asset: Asset,
    tokenAmount: bigint,
    localFeeAssetId: any,
    localFeeAmount: bigint,
    remoteFeeAssetId: any,
    remoteFeeAmount: bigint,
    topic: string
) {
    let beneficiaryLocation = parseLocation(beneficiary)
    let sourceLocation = parseLocation(sourceAccount)
    let tokenLocation = asset.location || erc20Location(ethChainId, asset.token)
    let assets = []
    if (JSON.stringify(localFeeAssetId) == JSON.stringify(tokenLocation)) {
        assets.push({
            id: localFeeAssetId,
            fun: {
                Fungible: localFeeAmount + tokenAmount,
            },
        })
        assets.push({
            id: remoteFeeAssetId,
            fun: {
                Fungible: remoteFeeAmount,
            },
        })
    } else {
        // native asset first
        if (tokenLocation.parents == 0) {
            assets.push({
                id: tokenLocation,
                fun: {
                    Fungible: tokenAmount,
                },
            })
        }
        assets.push({
            id: localFeeAssetId,
            fun: {
                Fungible: localFeeAmount,
            },
        })
        if (JSON.stringify(tokenLocation) == JSON.stringify(remoteFeeAssetId)) {
            assets.push({
                id: remoteFeeAssetId,
                fun: {
                    Fungible: tokenAmount + remoteFeeAmount,
                },
            })
        } else {
            if (tokenLocation.parents != 0) {
                assets.push({
                    id: tokenLocation,
                    fun: {
                        Fungible: tokenAmount,
                    },
                })
            }
            assets.push({
                id: remoteFeeAssetId,
                fun: {
                    Fungible: remoteFeeAmount,
                },
            })
        }
    }
    let transferredAsset = asset.location
        ? {
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
          }
        : {
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
          }
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                withdrawAsset: assets,
            },
            {
                payfees: {
                    asset: {
                        id: localFeeAssetId,
                        fun: {
                            Fungible: localFeeAmount,
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
                                    id: remoteFeeAssetId,
                                    fun: {
                                        Fungible: remoteFeeAmount,
                                    },
                                },
                            ],
                        },
                    },
                    preserveOrigin: true,
                    assets: [transferredAsset],
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
        ],
    })
}

export function buildTransferXcmFromParachain(
    registry: Registry,
    ethChainId: number,
    sourceAccount: string,
    beneficiary: string,
    assetHubParaId: number,
    sourceParachainId: number,
    asset: Asset,
    tokenAmount: bigint,
    localFeeAssetId: any,
    localFeeAmount: bigint,
    assethubFeeAssetId: any,
    assethubFeeAmount: bigint,
    remoteFeeAssetId: any,
    remoteFeeAmount: bigint,
    topic: string
) {
    let beneficiaryLocation = parseLocation(beneficiary)
    let sourceLocation = parseLocation(sourceAccount)
    let tokenLocation = asset.location || erc20Location(ethChainId, asset.token)
    let assets = []
    // native asset first
    if (tokenLocation.parents == 0) {
        assets.push({
            id: tokenLocation,
            fun: {
                Fungible: tokenAmount,
            },
        })
    }
    if (JSON.stringify(localFeeAssetId) == JSON.stringify(assethubFeeAssetId)) {
        assets.push({
            id: localFeeAssetId,
            fun: {
                Fungible: localFeeAmount + assethubFeeAmount,
            },
        })
    } else {
        assets.push({
            id: localFeeAssetId,
            fun: {
                Fungible: localFeeAmount,
            },
        })
        assets.push({
            id: assethubFeeAssetId,
            fun: {
                Fungible: assethubFeeAmount,
            },
        })
    }
    if (JSON.stringify(tokenLocation) == JSON.stringify(remoteFeeAssetId)) {
        assets.push({
            id: remoteFeeAssetId,
            fun: {
                Fungible: remoteFeeAmount + tokenAmount,
            },
        })
    } else {
        if (tokenLocation.parents != 0) {
            assets.push({
                id: tokenLocation,
                fun: {
                    Fungible: tokenAmount,
                },
            })
        }
        assets.push({
            id: remoteFeeAssetId,
            fun: {
                Fungible: remoteFeeAmount,
            },
        })
    }

    let transferredAsset = asset.location
        ? {
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
          }
        : {
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
          }
    let transferredAssetReanchored = asset.location
        ? {
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
          }
        : {
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
          }
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                withdrawAsset: assets,
            },
            {
                payfees: {
                    asset: {
                        id: localFeeAssetId,
                        fun: {
                            Fungible: localFeeAmount,
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
                    destination: parachainLocation(assetHubParaId),
                    remote_fees: {
                        reserveWithdraw: {
                            definite: [
                                {
                                    id: assethubFeeAssetId,
                                    fun: {
                                        Fungible: assethubFeeAmount,
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
                                        id: remoteFeeAssetId,
                                        fun: {
                                            Fungible: remoteFeeAmount,
                                        },
                                    },
                                ],
                            },
                        },
                        transferredAsset,
                    ],
                    remoteXcm: [
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
                                        beneficiary: parachainLocation(sourceParachainId),
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
                                                id: remoteFeeAssetId,
                                                fun: {
                                                    Fungible: remoteFeeAmount,
                                                },
                                            },
                                        ],
                                    },
                                },
                                preserveOrigin: true,
                                assets: [transferredAssetReanchored],
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
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}
