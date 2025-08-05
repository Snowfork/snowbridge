import { Registry } from "@polkadot/types/types"
import { erc20Location, ethereumNetwork, accountId32Location } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

export function buildAssetHubXcm(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    executionFee: bigint,
    value: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    topic: string
) {
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
    let reserveAssetDeposited = [
        {
            id: ether,
            fun: {
                Fungible: value,
            },
        },
    ]
    if (tokenAddress !== ETHER_TOKEN_ADDRESS) {
        reserveAssetDeposited.push({
            id: erc20Location(ethChainId, tokenAddress),
            fun: {
                Fungible: value,
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
                            Fungible: executionFee,
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
                            Fungible: executionFee,
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
                depositAsset: {
                    assets: {
                        wild: {
                            allCounted: 2,
                        },
                    },
                    beneficiary: accountId32Location(beneficiary),
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}
