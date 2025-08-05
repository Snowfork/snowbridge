import { Registry } from "@polkadot/types/types"
import {accountId32Location, erc20Location, ethereumNetwork} from "../../xcmBuilder"
import { beneficiaryMultiAddress } from "../../utils"
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

export function buildParachainERC20ReceivedXcmOnDestination(
    registry: Registry,
    ethChainId: number,
    tokenAddress: string,
    transferAmount: bigint,
    feeInEther: bigint,
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
