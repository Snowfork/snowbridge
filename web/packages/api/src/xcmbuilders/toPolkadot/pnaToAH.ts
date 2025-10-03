import { Registry } from "@polkadot/types/types"
import { erc20Location, ethereumNetwork, accountToLocation } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

export function buildAssetHubPNAReceivedXcm(
    registry: Registry,
    ethChainId: number,
    tokenLocation: any,
    etherAmount: bigint,
    totalAssetHubFeeInEther: bigint,
    tokenValue: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    topic: string
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
                            Fungible: tokenValue,
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
    })
}

export function sendMessageXCM(registry: Registry, beneficiary: string, topic: string) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    return registry.createType("XcmVersionedXcm", {
        v5: [
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
