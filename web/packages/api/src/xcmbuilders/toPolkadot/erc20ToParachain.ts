import { Registry } from "@polkadot/types/types"
import {
    accountToLocation,
    erc20Location,
    ethereumNetwork,
    parachainLocation,
    DOT_LOCATION,
} from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

export function buildAssetHubERC20ReceivedXcm(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    etherAmount: bigint,
    totalAssetHubFeeInEther: bigint,
    tokenAmount: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    destinationParaId: number,
    remoteEtherFeeAmount: bigint,
    topic: string
) {
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    let beneficiaryLocation = accountToLocation(beneficiary)
    let reserveAssetDeposited = []
    if (tokenAddress === ETHER_TOKEN_ADDRESS) {
        reserveAssetDeposited.push({
            id: ether,
            fun: {
                Fungible: tokenAmount + etherAmount,
            },
        })
    } else if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
        if (etherAmount > 0) {
            reserveAssetDeposited.push({
                id: ether,
                fun: {
                    Fungible: etherAmount,
                },
            })
        }
        reserveAssetDeposited.push({
            id: erc20Location(ethChainId, tokenAddress),
            fun: {
                Fungible: tokenAmount,
            },
        })
    }
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                descendOrigin: { x1: [{ PalletInstance: 91 }] },
            },
            {
                universalOrigin: ethereumNetwork(ethChainId),
            },
            {
                reserveAssetDeposited: [
                    {
                        id: ether,
                        fun: {
                            Fungible: totalAssetHubFeeInEther,
                        },
                    },
                ],
            },
            {
                setHints: {
                    hints: [{ assetClaimer: { location: claimer } }],
                },
            },
            {
                payFees: {
                    asset: {
                        id: ether,
                        fun: {
                            Fungible: totalAssetHubFeeInEther,
                        },
                    },
                },
            },
            {
                reserveAssetDeposited: reserveAssetDeposited,
            },
            {
                descendOrigin: {
                    x1: [
                        {
                            AccountKey20: {
                                key: origin,
                                network: null,
                            },
                        },
                    ],
                },
            },
            {
                initiateTransfer: {
                    destination: parachainLocation(destinationParaId),
                    remote_fees: {
                        reserveDeposit: {
                            definite: [
                                {
                                    id: ether,
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
                                        id: erc20Location(ethChainId, tokenAddress),
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
                depositAsset: {
                    assets: {
                        wild: {
                            allOf: { id: ether, fun: "Fungible" },
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
    })
}

export function buildParachainERC20ReceivedXcmOnDestination(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    transferAmount: bigint,
    feeInEther: bigint,
    beneficiary: string,
    topic: string
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    let reserveAssetDeposited = []
    if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
        reserveAssetDeposited.push({
            id: erc20Location(ethChainId, tokenAddress),
            fun: {
                Fungible: transferAmount,
            },
        })
    } else {
        reserveAssetDeposited.push({
            id: ether,
            fun: {
                Fungible: feeInEther + transferAmount,
            },
        })
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                reserveAssetDeposited: reserveAssetDeposited,
            },
            { clearOrigin: null },
            {
                buyExecution: {
                    fees: {
                        id: ether,
                        fun: {
                            Fungible: feeInEther,
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

export function sendMessageXCM(
    registry: Registry,
    ethChainId: number,
    destinationParaId: number,
    tokenAddress: string,
    beneficiary: string,
    tokenAmount: bigint,
    remoteEtherFeeAmount: bigint,
    topic: string
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                initiateTransfer: {
                    destination: parachainLocation(destinationParaId),
                    remote_fees: {
                        reserveDeposit: {
                            definite: [
                                {
                                    id: ether,
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
                                        id: erc20Location(ethChainId, tokenAddress),
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
                refundSurplus: null,
            },
            {
                depositAsset: {
                    assets: {
                        wild: {
                            allOf: { id: ether, fun: "Fungible" },
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
    })
}

export function buildParachainERC20ReceivedXcmOnDestWithDOTFee(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    etherAmount: bigint,
    totalAssetHubFeeInEther: bigint,
    tokenAmount: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    destinationParaId: number,
    remoteEtherFeeAmount: bigint,
    remoteDotFeeAmount: bigint,
    topic: string
) {
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    let beneficiaryLocation = accountToLocation(beneficiary)
    let reserveAssetDeposited = []
    if (tokenAddress === ETHER_TOKEN_ADDRESS) {
        reserveAssetDeposited.push({
            id: ether,
            fun: {
                Fungible: tokenAmount + etherAmount,
            },
        })
    } else if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
        if (etherAmount > 0) {
            reserveAssetDeposited.push({
                id: ether,
                fun: {
                    Fungible: etherAmount,
                },
            })
        }
        reserveAssetDeposited.push({
            id: erc20Location(ethChainId, tokenAddress),
            fun: {
                Fungible: tokenAmount,
            },
        })
    }
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                descendOrigin: { x1: [{ PalletInstance: 91 }] },
            },
            {
                universalOrigin: ethereumNetwork(ethChainId),
            },
            {
                reserveAssetDeposited: [
                    {
                        id: ether,
                        fun: {
                            Fungible: totalAssetHubFeeInEther,
                        },
                    },
                ],
            },
            {
                setHints: {
                    hints: [{ assetClaimer: { location: claimer } }],
                },
            },
            {
                payFees: {
                    asset: {
                        id: ether,
                        fun: {
                            Fungible: totalAssetHubFeeInEther,
                        },
                    },
                },
            },
            {
                reserveAssetDeposited: reserveAssetDeposited,
            },
            {
                descendOrigin: {
                    x1: [
                        {
                            AccountKey20: {
                                key: origin,
                                network: null,
                            },
                        },
                    ],
                },
            },
            {
                exchangeAsset: {
                    give: {
                        definite: [
                            {
                                id: ether,
                                fun: {
                                    Fungible: remoteDotFeeAmount,
                                },
                            },
                        ],
                    },
                    want: [
                        {
                            id: DOT_LOCATION,
                            fun: {
                                Fungible: remoteDotFeeAmount,
                            },
                        },
                    ],
                    maximal: true,
                },
            },
            {
                initiateTransfer: {
                    destination: parachainLocation(destinationParaId),
                    remote_fees: {
                        reserveDeposit: {
                            definite: [
                                {
                                    id: ether,
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
                                        id: erc20Location(ethChainId, tokenAddress),
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
                depositAsset: {
                    assets: {
                        wild: {
                            allOf: { id: ether, fun: "Fungible" },
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
    })
}

export function sendMessageXCMWithDOTDestFee(
    registry: Registry,
    ethChainId: number,
    destinationParaId: number,
    tokenAddress: string,
    beneficiary: string,
    tokenAmount: bigint,
    remoteEtherFeeAmount: bigint,
    remoteDotFeeAmount: bigint,
    topic: string
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                exchangeAsset: {
                    give: {
                        definite: [
                            {
                                id: ether,
                                fun: {
                                    Fungible: remoteDotFeeAmount,
                                },
                            },
                        ],
                    },
                    want: [
                        {
                            id: DOT_LOCATION,
                            fun: {
                                Fungible: remoteDotFeeAmount,
                            },
                        },
                    ],
                    maximal: true,
                },
            },
            {
                initiateTransfer: {
                    destination: parachainLocation(destinationParaId),
                    remote_fees: {
                        reserveDeposit: {
                            definite: [
                                {
                                    id: ether,
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
                                        id: erc20Location(ethChainId, tokenAddress),
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
                refundSurplus: null,
            },
            {
                depositAsset: {
                    assets: {
                        wild: {
                            allOf: { id: ether, fun: "Fungible" },
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
    })
}
