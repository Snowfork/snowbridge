import { AssetMap, PNAMap } from "../types"
import { ParachainBase } from "./parachainBase"
import { DOT_LOCATION, getTokenFromLocation } from "../xcmBuilder"

export const WESTEND_GENESIS = "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e"

export class AssetHubParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint> {
        let accountData: any
        if (pnaAssetId) {
            accountData = (
                await this.provider.query.assets.account(pnaAssetId, account)
            ).toPrimitive() as any
        } else {
            accountData = (
                await this.provider.query.foreignAssets.account(location, account)
            ).toPrimitive() as any
        }
        return BigInt(accountData?.balance ?? 0n)
    }

    getDotBalance(account: string): Promise<bigint> {
        return this.getNativeBalance(account)
    }

    getAssets(ethChainId: number, pnas: PNAMap): Promise<AssetMap> {
        return this.getAssetsFiltered(ethChainId, pnas, bridgeablePNAsOnAH)
    }

    async getAssetsFiltered(
        ethChainId: number,
        pnas: PNAMap,
        pnaFilter: (location: any, assetHubParaId: number) => any
    ) {
        const assets: AssetMap = {}
        // ERC20
        {
            const entries = await this.provider.query.foreignAssets.asset.entries()
            for (const [key, value] of entries) {
                const location: any = key.args.at(0)?.toJSON()
                if (!location) {
                    console.warn(
                        `Could not convert ${key.toHuman()} to location for ${this.specName}.`
                    )
                    continue
                }
                const token = getTokenFromLocation(location, ethChainId)
                if (!token) {
                    continue
                }

                const asset: any = value.toJSON()
                const assetMetadata: any = (
                    await this.provider.query.foreignAssets.metadata(location)
                ).toPrimitive()

                assets[token] = {
                    token,
                    name: String(assetMetadata.name),
                    minimumBalance: BigInt(asset.minBalance),
                    symbol: String(assetMetadata.symbol),
                    decimals: Number(assetMetadata.decimals),
                    isSufficient: Boolean(asset.isSufficient),
                }
            }
        }
        // PNA
        {
            for (const { token, foreignId, ethereumlocation } of Object.keys(pnas).map(
                (p) => pnas[p]
            )) {
                const locationOnAH: any = pnaFilter(ethereumlocation, this.parachainId)
                if (!locationOnAH) {
                    console.warn(
                        `Location ${JSON.stringify(ethereumlocation)} is not bridgeable on ${
                            this.specName
                        }`
                    )
                    continue
                }

                if (locationOnAH?.parents == 0) {
                    const assetId = locationOnAH?.interior?.x2[1]?.generalIndex
                    const [assetInfo, assetMeta] = (
                        await Promise.all([
                            this.provider.query.assets.asset(assetId),
                            this.provider.query.assets.metadata(assetId),
                        ])
                    ).map((encoded) => encoded.toPrimitive() as any)

                    assets[token.toLowerCase()] = {
                        token: token.toLowerCase(),
                        name: String(assetMeta.name),
                        symbol: String(assetMeta.symbol),
                        decimals: Number(assetMeta.decimals),
                        locationOnEthereum: ethereumlocation,
                        location: locationOnAH,
                        locationOnAH,
                        foreignId: foreignId,
                        minimumBalance: BigInt(assetInfo?.minBalance),
                        isSufficient: Boolean(assetInfo?.isSufficient),
                        assetId: String(assetId),
                    }
                } else if (
                    locationOnAH?.parents == DOT_LOCATION.parents &&
                    locationOnAH?.interior == DOT_LOCATION.interior
                ) {
                    let existentialDeposit =
                        this.provider.consts.balances.existentialDeposit.toPrimitive()
                    const chainInfo = await this.chainProperties()

                    assets[token.toLowerCase()] = {
                        token: token.toLowerCase(),
                        name: "",
                        symbol: String(chainInfo.tokenSymbols),
                        decimals: Number(chainInfo.tokenDecimals),
                        locationOnEthereum: ethereumlocation,
                        location: locationOnAH,
                        locationOnAH,
                        foreignId: foreignId,
                        minimumBalance: BigInt(existentialDeposit as any),
                        isSufficient: true,
                    }
                } else {
                    const assetType = this.provider.registry.createType(
                        "StagingXcmV4Location",
                        locationOnAH
                    )
                    const [assetInfo, assetMeta] = (
                        await Promise.all([
                            this.provider.query.foreignAssets.asset(assetType),
                            this.provider.query.foreignAssets.metadata(assetType),
                        ])
                    ).map((encoded) => encoded.toPrimitive() as any)
                    if (!assetInfo) {
                        console.warn(
                            `Asset '${JSON.stringify(
                                locationOnAH
                            )}' is not a registered foregin asset on ${this.specName}.`
                        )
                        continue
                    }

                    assets[token.toLowerCase()] = {
                        token: token.toLowerCase(),
                        name: String(assetMeta.name),
                        symbol: String(assetMeta.symbol),
                        decimals: Number(assetMeta.decimals),
                        locationOnEthereum: ethereumlocation,
                        location: locationOnAH,
                        locationOnAH,
                        foreignId: foreignId,
                        minimumBalance: BigInt(assetInfo?.minBalance),
                        isSufficient: Boolean(assetInfo?.isSufficient),
                    }
                }
            }
        }
        return assets
    }
}

// Currently, the bridgeable assets are limited to KSM, DOT, native assets on AH
// and TEER
function bridgeablePNAsOnAH(location: any, assetHubParaId: number): any {
    if (location.parents != 1) {
        return
    }
    // KSM
    if (location.interior.x1 && location.interior.x1[0]?.globalConsensus?.kusama !== undefined) {
        return {
            parents: 2,
            interior: {
                x1: [
                    {
                        globalConsensus: {
                            kusama: null,
                        },
                    },
                ],
            },
        }
    }
    // DOT
    else if (
        location.interior.x1 &&
        location.interior.x1[0]?.globalConsensus?.polkadot !== undefined
    ) {
        return DOT_LOCATION
    }
    // Native assets from AH
    else if (
        location.interior.x4 &&
        location.interior.x4[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x4[1]?.parachain == assetHubParaId
    ) {
        return {
            parents: 0,
            interior: {
                x2: [
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
            parents: 1,
            interior: {
                x1: [
                    {
                        parachain: 2039,
                    },
                ],
            },
        }
    }
    // Add assets for Westend
    if (
        location.interior.x1 &&
        location.interior.x1[0]?.globalConsensus?.byGenesis === WESTEND_GENESIS
    ) {
        return DOT_LOCATION
    } else if (
        location.interior.x2 &&
        location.interior.x2[0]?.globalConsensus?.byGenesis === WESTEND_GENESIS &&
        location.interior.x2[1]?.parachain != undefined
    ) {
        return {
            parents: 1,
            interior: {
                x1: [
                    {
                        parachain: location.interior.x2[1]?.parachain,
                    },
                ],
            },
        }
    }
}
