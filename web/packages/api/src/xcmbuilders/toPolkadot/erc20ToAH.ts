import { Registry } from "@polkadot/types/types"
import { erc20Location, ethereumNetwork, accountToLocation } from "../../xcmBuilder"
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
    topic: string,
    customXcm?: any[]
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
            {
                setTopic: topic,
            },
        ],
    })
}

export function sendMessageXCM(registry: Registry, beneficiary: string, topic: string, customXcm?: any[]) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                refundSurplus: null,
            },
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
            {
                setTopic: topic,
            },
        ],
    })
}
