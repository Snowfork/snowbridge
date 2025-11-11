import { Registry } from "@polkadot/types/types"
import {
    accountToLocation,
    erc20Location,
    ethereumNetwork,
    parachainLocation,
} from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

export function buildAssetHubPNAReceivedXcm(
    registry: Registry,
    ethChainId: number,
    tokenLocation: any,
    etherAmount: bigint,
    totalAssetHubFeeInEther: bigint,
    remoteExecutionFee: bigint,
    value: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    destinationParaId: number,
    topic: string,
    customXcm?: any[],
) {
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    let beneficiaryLocation = accountToLocation(beneficiary)
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
                reserveAssetDeposited: [
                    {
                        id: ether,
                        fun: {
                            Fungible: etherAmount,
                        },
                    },
                ],
            },
            {
                withdrawAsset: [
                    {
                        id: tokenLocation,
                        fun: {
                            Fungible: value,
                        },
                    },
                ],
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
                                        Fungible: remoteExecutionFee,
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
                                            Fungible: value,
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
                        ...(customXcm || []), // Insert custom XCM instructions if provided
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

export function buildParachainPNAReceivedXcmOnDestination(
    registry: Registry,
    ethChainId: number,
    assetLocation: any,
    transferAmount: bigint,
    feeInEther: bigint,
    beneficiary: string,
    topic: string,
    customXcm?: any[],
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                reserveAssetDeposited: [
                    {
                        id: ether,
                        fun: {
                            Fungible: feeInEther,
                        },
                    },
                ],
            },
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
            ...(customXcm || []), // Insert custom XCM instructions if provided
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
    tokenLocation: any,
    beneficiary: string,
    tokenAmount: bigint,
    remoteEtherFeeAmount: bigint,
    topic: string,
    customXcm?: any[],
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
                                        id: tokenLocation,
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
                        ...(customXcm || []), // Insert custom XCM instructions if provided
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
