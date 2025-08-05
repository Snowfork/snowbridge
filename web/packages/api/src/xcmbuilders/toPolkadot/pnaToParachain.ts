import { Registry } from "@polkadot/types/types"
import {accountId32Location, erc20Location, ethereumNetwork} from "../../xcmBuilder"
import { beneficiaryMultiAddress } from "../../utils"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"

export function buildAssetHubXcm(
    registry: Registry,
    ethChainId: number,
    tokenLocation: any,
    executionFee: bigint,
    value: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    topic: string
) {
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS)
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


export function buildParachainPNAReceivedXcmOnDestination(
    registry: Registry,
    ethChainId: number,
    assetLocation: any,
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
