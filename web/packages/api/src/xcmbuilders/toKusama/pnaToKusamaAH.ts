import { Registry } from "@polkadot/types/types"
import {
    accountToLocation,
    erc20Location,
    ethereumNetwork,
    kusamaAssetHubLocation,
} from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

/**
 * Builds the full XCM that will execute on Polkadot AssetHub when PNA tokens arrive from Ethereum,
 * then forwards them to Kusama AssetHub via initiateTransfer.
 *
 * PNA uses withdrawAsset instead of reserveAssetDeposited for the token (since AH is the reserve).
 */
export function buildAssetHubPNAReceivedXcmForKusama(
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
    kusamaAHParaId: number,
    topic: string,
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
                    destination: kusamaAssetHubLocation(kusamaAHParaId),
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
                    preserveOrigin: false,
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

/**
 * Builds the inner XCM for PNA that is encoded in the gateway v2_sendMessage call.
 */
export function sendMessageXCM(
    registry: Registry,
    ethChainId: number,
    kusamaAHParaId: number,
    tokenLocation: any,
    beneficiary: string,
    tokenAmount: bigint,
    remoteEtherFeeAmount: bigint,
    topic: string,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)

    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                initiateTransfer: {
                    destination: kusamaAssetHubLocation(kusamaAHParaId),
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
                    preserveOrigin: false,
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
