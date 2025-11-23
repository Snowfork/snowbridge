import { AssetMap } from "@snowbridge/base-types"
import { PNAMap } from "../assets_v2"
import { ParachainBase } from "./parachainBase"
import { DOT_LOCATION, getTokenFromLocation, WESTEND_GENESIS } from "../xcmBuilder"

export class PenpalParachain extends ParachainBase {
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
        return this.getLocationBalance(DOT_LOCATION, account)
    }

    getAssets(ethChainId: number, pnas: PNAMap): Promise<AssetMap> {
        return this.getAssetsFiltered(ethChainId, pnas, bridgeablePNAsOnPenpal)
    }

    async getAssetsFiltered(
        ethChainId: number,
        pnas: PNAMap,
        pnaFilter: (location: any, paraId: number, env: string) => any,
    ) {
        const assets: AssetMap = {}
        // ERC20
        {
            const entries = await this.provider.query.foreignAssets.asset.entries()
            for (const [key, value] of entries) {
                const location: any = key.args[0]?.toJSON()
                if (!location) {
                    console.warn(
                        `Could not convert ${key.toHuman()} to location for ${this.specName}.`,
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
                (p) => pnas[p],
            )) {
                const locationPair: any = pnaFilter(
                    ethereumlocation,
                    this.parachainId,
                    this.specName,
                )
                if (!locationPair) {
                    console.warn(
                        `Location ${JSON.stringify(ethereumlocation)} is not bridgeable on ${
                            this.specName
                        }`,
                    )
                    continue
                }
                let locationOnChain = locationPair?.local

                if (locationOnChain?.parents == 0) {
                    const assetId = locationOnChain?.interior?.x2[1]?.generalIndex
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
                        location: locationOnChain,
                        locationOnAH: locationPair?.assethub,
                        foreignId: foreignId,
                        minimumBalance: BigInt(assetInfo?.minBalance),
                        isSufficient: Boolean(assetInfo?.isSufficient),
                        assetId: String(assetId),
                    }
                }
            }
        }
        return assets
    }
}

function bridgeablePNAsOnPenpal(location: any, assetHubParaId: number, env: string): any {
    if (location.parents != 1) {
        return
    }
    switch (env) {
        case "penpal-parachain":
            // Add assets for Westend
            if (
                location.interior.x4 &&
                location.interior.x4[0]?.globalConsensus?.byGenesis === WESTEND_GENESIS &&
                location.interior.x4[1]?.parachain &&
                location.interior.x4[2]?.palletInstance &&
                location.interior.x4[3]?.generalIndex != undefined
            ) {
                return {
                    local: {
                        parents: 0,
                        interior: {
                            x2: [
                                { palletInstance: location.interior.x4[2].palletInstance },
                                { generalIndex: location.interior.x4[3].generalIndex },
                            ],
                        },
                    },
                    assethub: {
                        parents: 1,
                        interior: {
                            x3: [
                                { parachain: location.interior.x4[1]?.parachain },
                                { palletInstance: location.interior.x4[2].palletInstance },
                                { generalIndex: location.interior.x4[3].generalIndex },
                            ],
                        },
                    },
                }
            }
    }
}
