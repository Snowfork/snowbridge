import { Registry } from "@polkadot/types/types"
import {
    accountToLocation,
    erc20Location,
    ethereumNetwork,
    kusamaAssetHubLocation,
    DOT_LOCATION,
} from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

/**
 * Builds the full XCM that will execute on Polkadot AssetHub when tokens arrive from Ethereum,
 * then forwards them to Kusama AssetHub via initiateTransfer.
 *
 * Flow: BridgeHub delivers to Polkadot AH → AH executes this XCM → initiateTransfer to Kusama AH
 */
export function buildAssetHubERC20ReceivedXcmForKusama(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    etherAmount: bigint,
    totalAssetHubFeeInEther: bigint,
    tokenAmount: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    kusamaAHParaId: number,
    remoteEtherFeeAmount: bigint,
    topic: string,
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
    } else {
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

/**
 * Builds the inner XCM that is encoded in the gateway v2_sendMessage call.
 * This XCM executes on Polkadot AH and initiates transfer to Kusama AH.
 */
export function sendMessageXCM(
    registry: Registry,
    ethChainId: number,
    kusamaAHParaId: number,
    tokenAddress: string,
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
