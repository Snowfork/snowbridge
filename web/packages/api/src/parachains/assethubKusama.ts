import { AssetMap, PNAMap } from "@snowbridge/base-types"
import { dotLocationOnKusamaAssetHub } from "../xcmBuilder"
import { AssetHubParachain } from "./assethub"

export class AssetHubKusamaParachain extends AssetHubParachain {
    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(dotLocationOnKusamaAssetHub, account)
    }
    async getAssets(ethChainId: number, pnas: PNAMap): Promise<AssetMap> {
        return this.getAssetsFiltered(
            ethChainId,
            bridgeableENAsOnAH,
            pnas,
            bridgeablePNAsOnKusamaAH,
        )
    }

    async swapAsset1ForAsset2(asset1: any, asset2: any, exactAsset1Balance: bigint) {
        const result = await this.provider.call.assetConversionApi.quotePriceExactTokensForTokens(
            asset1,
            asset2,
            exactAsset1Balance,
            true,
        )
        const asset2Balance = result.toPrimitive() as any
        if (asset2Balance == null) {
            throw Error(
                `No pool set up in asset conversion pallet for '${JSON.stringify(
                    asset1,
                )}' and '${JSON.stringify(asset2)}'.`,
            )
        }
        return BigInt(asset2Balance)
    }

    async getAssetHubConversionPalletSwap(
        asset1: any,
        asset2: any,
        exactAsset2Balance: bigint,
    ): Promise<bigint> {
        const result = await this.provider.call.assetConversionApi.quotePriceTokensForExactTokens(
            asset1,
            asset2,
            exactAsset2Balance,
            true,
        )
        const asset1Balance = result.toPrimitive() as any
        if (asset1Balance == null) {
            throw Error(
                `No pool set up in asset conversion pallet for '${JSON.stringify(
                    asset1,
                )}' and '${JSON.stringify(asset2)}'.`,
            )
        }
        return BigInt(asset1Balance)
    }
}

function bridgeablePNAsOnKusamaAH(location: any, assetHubParaId: number): any {
    if (location.parents != 1) {
        return
    }
    // KSM
    if (location.interior.x1 && location.interior.x1[0]?.globalConsensus?.kusama !== undefined) {
        return {
            parents: 1,
            interior: "Here",
        }
    }
    // DOT
    else if (
        location.interior.x1 &&
        location.interior.x1[0]?.globalConsensus?.polkadot !== undefined
    ) {
        return {
            parents: 2,
            interior: {
                x1: [
                    {
                        globalConsensus: {
                            Polkadot: null,
                        },
                    },
                ],
            },
        }
    }
    // Native assets from AH
    else if (
        location.interior.x4 &&
        location.interior.x4[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x4[1]?.parachain == assetHubParaId
    ) {
        return {
            parents: 2,
            interior: {
                x4: [
                    {
                        globalConsensus: {
                            Polkadot: null,
                        },
                    },
                    {
                        parachain: assetHubParaId,
                    },
                    {
                        palletInstance: location.interior.x4[2]?.palletInstance,
                    },
                    {
                        generalIndex: location.interior.x4[3]?.generalIndex,
                    },
                ],
            },
        }
    }
    // Others from 3rd Parachains, only TEER for now
    else if (
        location.interior.x2 &&
        location.interior.x2[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x2[1]?.parachain == 2039
    ) {
        return {
            parents: 2,
            interior: {
                x2: [
                    {
                        globalConsensus: {
                            Polkadot: null,
                        },
                    },
                    {
                        parachain: 2039,
                    },
                ],
            },
        }
    }
}

// MYTH token is not transferable between Polkadot and Kusama AH.
function bridgeableENAsOnAH(token: string): boolean {
    return token != "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003"
}
