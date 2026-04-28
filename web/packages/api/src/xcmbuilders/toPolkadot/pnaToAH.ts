import { Registry } from "@polkadot/types/types"
import {
    accountToLocation,
    buildSplitDepositAsset,
    erc20Location,
    ethereumNetwork,
} from "../../xcmBuilder"
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
            // Mirror the user-side `sendMessageXCM` tail exactly: RefundSurplus
            // returns unused PayFees ether to holding so the subsequent
            // DepositAsset attempts to settle ether dust + tokens together.
            // Without this, the dry-run misses ether-dust BelowMinimum traps.
            { refundSurplus: null },
            ...(customXcm || []), // Insert custom XCM instructions if provided
            ...buildSplitDepositAsset(beneficiaryLocation, tokenLocation, 2),
            {
                setTopic: topic,
            },
        ],
    })
}

export function sendMessageXCM(
    registry: Registry,
    beneficiary: string,
    topic: string,
    customXcm?: any[],
    userAssetLocation?: any,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                refundSurplus: null,
            },
            ...(customXcm || []), // Insert custom XCM instructions if provided
            ...buildSplitDepositAsset(beneficiaryLocation, userAssetLocation, 2),
            {
                setTopic: topic,
            },
        ],
    })
}
