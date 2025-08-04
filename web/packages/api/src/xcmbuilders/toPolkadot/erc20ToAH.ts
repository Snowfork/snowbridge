import { Registry } from "@polkadot/types/types"
import {
    bridgeLocation,
    DOT_LOCATION,
    erc20Location,
    erc20LocationReanchored,
    accountToLocation,
    HERE_LOCATION,
    isEthereumNative, ethereumNetwork,
} from "../../xcmBuilder"
import { Asset } from "@snowbridge/base-types"
import {beneficiaryMultiAddress} from "../../utils";
import {ETHER_TOKEN_ADDRESS} from "../../assets_v2";

export function buildAssetHubXcm(
    registry: Registry,
    ethChainId: number,
    executionFee: bigint,
    claimer: any,
    exe: any,
) {
    let ether = erc20Location(ethChainId, ETHER_TOKEN_ADDRESS);
    let instructions =  registry.createType("XcmVersionedXcm", {
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
                setHints: [
                    {
                        hints: [ {
                            assetClaimer: {
                                location: claimer,
                            }
                        }]
                    },
                ],
            },
            {
                payFees: {
                    asset: {
                        id: ether,
                        fun: {
                            Fungible: localDOTFeeAmount,
                        },
                    },
                },
            },
        ],
    })
}
