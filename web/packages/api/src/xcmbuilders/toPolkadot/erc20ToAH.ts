import { Registry } from "@polkadot/types/types"
import { erc20Location, ethereumNetwork, accountId32Location, DOT_LOCATION } from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { beneficiaryMultiAddress } from "../../utils"

export function buildAssetHubERC20ReceivedXcm(
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

export function sendMessageXCM(registry: Registry, beneficiary: string, topic: string) {
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
                refundSurplus: null,
            },
            {
                depositAsset: {
                    assets: {
                        wild: {
                            allCounted: 2,
                        },
                    },
                    beneficiary: beneficiaryLocation,
                },
            },
            {
                setTopic: topic,
            },
        ],
    })
}
