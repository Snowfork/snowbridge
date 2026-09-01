import { Registry } from "@polkadot/types/types"
import {
    accountToLocation,
    buildPayloadEtherDeposit,
    buildServiceFeeDeposit,
    buildSplitDepositAsset,
    erc20Location,
    ethereumNetwork,
} from "../../xcmBuilder"
import { ETHER_TOKEN_ADDRESS } from "../../assets_v2"
import { ServiceFee } from "../../types/fee"

export function buildAssetHubPNAReceivedXcm(
    registry: Registry,
    ethChainId: number,
    tokenLocation: any,
    totalAssetHubFeeInEther: bigint,
    tokenValue: bigint,
    claimer: any,
    origin: string,
    beneficiary: string,
    topic: string,
    customXcm?: any[],
    serviceFee?: ServiceFee,
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
                withdrawAsset: [
                    {
                        id: tokenLocation,
                        fun: {
                            Fungible: tokenValue,
                        },
                    },
                ],
            },
            // The serviceFee arrives as payload ether on top of the fee ether.
            ...buildPayloadEtherDeposit(ether, serviceFee?.amount ?? 0n),
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
            // Before customXcm: the Definite take is saturating, so instructions a
            // caller supplies could drain holding and silently shrink the deposit.
            ...buildServiceFeeDeposit(ether, serviceFee),
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
    ethChainId: number,
    beneficiary: string,
    topic: string,
    customXcm?: any[],
    userAssetLocation?: any,
    serviceFee?: ServiceFee,
) {
    let beneficiaryLocation = accountToLocation(beneficiary)
    return registry.createType("XcmVersionedXcm", {
        v5: [
            {
                refundSurplus: null,
            },
            // Before customXcm: the Definite take is saturating, so instructions a
            // caller supplies could drain holding and silently shrink the deposit.
            ...buildServiceFeeDeposit(erc20Location(ethChainId, ETHER_TOKEN_ADDRESS), serviceFee),
            ...(customXcm || []), // Insert custom XCM instructions if provided
            ...buildSplitDepositAsset(beneficiaryLocation, userAssetLocation, 2),
            {
                setTopic: topic,
            },
        ],
    })
}
