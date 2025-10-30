import { Registry } from "@polkadot/types/types"
import {
    accountId32Location,
    dotLocationOnKusamaAssetHub,
    HERE_LOCATION,
    isDOTOnOtherConsensusSystem,
    isEthereumAsset,
    isKSMOnOtherConsensusSystem,
    isRelaychainLocation,
    ksmLocationOnPolkadotAssetHub,
    kusamaNetwork,
    NATIVE_TOKEN_LOCATION,
    parachainLocation,
    polkadotNetwork,
} from "./xcmBuilder"

export function buildTransferKusamaToPolkadotExportXCM(
    registry: Registry,
    transferTokenLocation: any,
    totalFeeInNative: bigint,
    feeOnDest: bigint,
    sourceAssetHubParaId: number,
    destAssetHubParaId: number,
    transferAmount: bigint,
    beneficiary: string,
    topic: string,
) {
    let withdrawAssetsOnSource: any[] = [
        {
            id: HERE_LOCATION,
            fun: {
                Fungible: totalFeeInNative,
            },
        },
    ]
    let reserveAssetsDepositedDest = [
        {
            id: ksmLocationOnPolkadotAssetHub,
            fun: {
                Fungible: totalFeeInNative,
            },
        },
    ]
    let withdrawAssetsDest: any[] = []

    if (isRelaychainLocation(transferTokenLocation)) {
        // If the asset transferred is KSM, only add the transfer amount to the asset
        withdrawAssetsOnSource[0].fun.Fungible =
            withdrawAssetsOnSource[0].fun.Fungible + transferAmount
        reserveAssetsDepositedDest[0].fun.Fungible =
            reserveAssetsDepositedDest[0].fun.Fungible + transferAmount
    } else if (isDOTOnOtherConsensusSystem(transferTokenLocation)) {
        // If the asset transferred is DOT, reanchor to KAH
        withdrawAssetsDest = [
            {
                id: NATIVE_TOKEN_LOCATION,
                fun: {
                    Fungible: transferAmount,
                },
            },
        ]
    } else if (isEthereumAsset(transferTokenLocation)) {
        // If the asset transferred is Ether or an ERC-20, the token location is already correct.
        withdrawAssetsDest.push({
            id: transferTokenLocation,
            fun: {
                Fungible: transferAmount,
            },
        })
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                withdrawAsset: withdrawAssetsOnSource,
            },
            {
                buyExecution: {
                    fees: {
                        id: HERE_LOCATION,
                        fun: {
                            Fungible: totalFeeInNative,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                setAppendix: [
                    {
                        depositAsset: {
                            assets: {
                                wild: {
                                    allCounted: 1,
                                },
                            },
                            beneficiary: parachainLocation(sourceAssetHubParaId),
                        },
                    },
                ],
            },
            {
                exportMessage: {
                    network: { Polkadot: { network: null } },
                    destination: { x1: [{ parachain: destAssetHubParaId }] },
                    xcm: [
                        {
                            reserveAssetDeposited: reserveAssetsDepositedDest,
                        },
                        {
                            buyExecution: {
                                fees: {
                                    id: ksmLocationOnPolkadotAssetHub,
                                    fun: {
                                        Fungible: feeOnDest,
                                    },
                                },
                                weight_limit: "Unlimited",
                            },
                        },
                        {
                            withdrawAsset: withdrawAssetsDest,
                        },
                        { clearOrigin: null },
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
                },
            },

            {
                setTopic: topic,
            },
        ],
    })
}

export function buildPolkadotToKusamaDestAssetHubXCM(
    registry: Registry,
    totalFeeInNative: bigint,
    assetHubParaId: number,
    transferTokenLocation: any,
    transferAmount: bigint,
    beneficiary: string,
    topic: string,
) {
    let withdrawAssets: any[] = []
    let reserveAssetsDeposited = [
        {
            id: dotLocationOnKusamaAssetHub,
            fun: {
                Fungible: totalFeeInNative,
            },
        },
    ]
    if (isRelaychainLocation(transferTokenLocation)) {
        // If the asset transferred is DOT, only add the transfer amount to the asset
        reserveAssetsDeposited[0].fun.Fungible =
            reserveAssetsDeposited[0].fun.Fungible + transferAmount
    } else if (isKSMOnOtherConsensusSystem(transferTokenLocation)) {
        // If the asset transferred is KSM, reanchor to KAH
        withdrawAssets = [
            {
                id: NATIVE_TOKEN_LOCATION,
                fun: {
                    Fungible: totalFeeInNative,
                },
            },
        ]
    } else if (isEthereumAsset(transferTokenLocation)) {
        // If the asset transferred is Ether or an ERC-20, the token location is already correct.
        reserveAssetsDeposited.push({
            id: transferTokenLocation,
            fun: {
                Fungible: transferAmount,
            },
        })
    }

    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                descendOrigin: { x1: [{ PalletInstance: 53 }] },
            },
            {
                universalOrigin: polkadotNetwork,
            },
            {
                descendOrigin: { x1: [{ parachain: assetHubParaId }] },
            },
            {
                reserveAssetDeposited: reserveAssetsDeposited,
            },
            {
                buyExecution: {
                    fees: {
                        id: dotLocationOnKusamaAssetHub,
                        fun: {
                            Fungible: totalFeeInNative,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                withdrawAsset: withdrawAssets,
            },
            { clearOrigin: null },
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

export function buildKusamaToPolkadotDestAssetHubXCM(
    registry: Registry,
    totalFeeInNative: bigint,
    assetHubParaId: number,
    transferTokenLocation: any,
    transferAmount: bigint,
    beneficiary: string,
    topic: string,
) {
    let withdrawAssets: any[] = []
    let reserveAssetsDeposited = [
        {
            id: ksmLocationOnPolkadotAssetHub,
            fun: {
                Fungible: totalFeeInNative,
            },
        },
    ]
    if (isRelaychainLocation(transferTokenLocation)) {
        // If the asset transferred is KSM, only add the transfer amount to the asset
        reserveAssetsDeposited[0].fun.Fungible =
            reserveAssetsDeposited[0].fun.Fungible + transferAmount
    } else if (isDOTOnOtherConsensusSystem(transferTokenLocation)) {
        // If the asset transferred is DOT, reanchor to PAH
        withdrawAssets = [
            {
                id: NATIVE_TOKEN_LOCATION,
                fun: {
                    Fungible: transferAmount,
                },
            },
        ]
    } else if (isEthereumAsset(transferTokenLocation)) {
        // If the asset transferred is Ether or an ERC-20, the token location is already correct.
        withdrawAssets = [
            {
                id: transferTokenLocation,
                fun: {
                    Fungible: transferAmount,
                },
            },
        ]
    }

    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                descendOrigin: { x1: [{ PalletInstance: 53 }] },
            },
            {
                universalOrigin: kusamaNetwork,
            },
            {
                descendOrigin: { x1: [{ parachain: assetHubParaId }] },
            },
            {
                reserveAssetDeposited: reserveAssetsDeposited,
            },
            {
                buyExecution: {
                    fees: {
                        id: ksmLocationOnPolkadotAssetHub,
                        fun: {
                            Fungible: totalFeeInNative,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                withdrawAsset: withdrawAssets,
            },
            { clearOrigin: null },
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

export function buildTransferPolkadotToKusamaExportXCM(
    registry: Registry,
    transferTokenLocation: any,
    totalFeeInNative: bigint,
    feeOnDest: bigint,
    sourceAssetHubParaId: number,
    destAssetHubParaId: number,
    transferAmount: bigint,
    beneficiary: string,
    topic: string,
) {
    let withdrawAssetsOnSource: any[] = [
        {
            id: HERE_LOCATION,
            fun: {
                Fungible: totalFeeInNative,
            },
        },
    ]
    let reserveAssetsDepositedDest = [
        {
            id: dotLocationOnKusamaAssetHub,
            fun: {
                Fungible: totalFeeInNative,
            },
        },
    ]
    let withdrawAssetsDest: any[] = []

    if (isRelaychainLocation(transferTokenLocation)) {
        // If the asset transferred is DOT, only add the transfer amount to the asset
        withdrawAssetsOnSource[0].fun.Fungible =
            withdrawAssetsOnSource[0].fun.Fungible + transferAmount
        reserveAssetsDepositedDest[0].fun.Fungible =
            reserveAssetsDepositedDest[0].fun.Fungible + transferAmount
    } else if (isKSMOnOtherConsensusSystem(transferTokenLocation)) {
        // If the asset transferred is KSM, reanchor to KAH
        withdrawAssetsDest = [
            {
                id: NATIVE_TOKEN_LOCATION,
                fun: {
                    Fungible: transferAmount,
                },
            },
        ]
    } else if (isEthereumAsset(transferTokenLocation)) {
        // If the asset transferred is Ether or an ERC-20, the token location is already correct.
        reserveAssetsDepositedDest.push({
            id: transferTokenLocation,
            fun: {
                Fungible: transferAmount,
            },
        })
    }
    return registry.createType("XcmVersionedXcm", {
        v4: [
            {
                withdrawAsset: withdrawAssetsOnSource,
            },
            {
                buyExecution: {
                    fees: {
                        id: HERE_LOCATION,
                        fun: {
                            Fungible: totalFeeInNative,
                        },
                    },
                    weightLimit: "Unlimited",
                },
            },
            {
                setAppendix: [
                    {
                        depositAsset: {
                            assets: {
                                wild: {
                                    allCounted: 1,
                                },
                            },
                            beneficiary: parachainLocation(sourceAssetHubParaId),
                        },
                    },
                ],
            },
            {
                exportMessage: {
                    network: { Kusama: { network: null } },
                    destination: { x1: [{ parachain: destAssetHubParaId }] },
                    xcm: [
                        {
                            reserveAssetDeposited: reserveAssetsDepositedDest,
                        },
                        {
                            withdrawAsset: withdrawAssetsDest,
                        },
                        { clearOrigin: null },
                        {
                            buyExecution: {
                                fees: {
                                    id: dotLocationOnKusamaAssetHub,
                                    fun: {
                                        Fungible: feeOnDest,
                                    },
                                },
                                weight_limit: "Unlimited",
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
                },
            },

            {
                setTopic: topic,
            },
        ],
    })
}
