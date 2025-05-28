import { AssetMap, PNAMap } from "src/assets_v2"
import { ParachainBase } from "./parachainBase"
import { DOT_LOCATION, getTokenFromLocation } from "../xcmBuilder"

export class BifrostParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        const paraAssetId = (
            await this.provider.query.assetRegistry.locationToCurrencyIds(location)
        ).toPrimitive()
        if (!paraAssetId) {
            throw Error(`'${JSON.stringify(location)}' not registered for spec ${this.specName}.`)
        }
        const accountData = (
            await this.provider.query.tokens.accounts(account, paraAssetId)
        ).toPrimitive() as any
        return BigInt(accountData?.free ?? 0n)
    }

    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(DOT_LOCATION, account)
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        const entries = await this.provider.query.assetRegistry.currencyIdToLocations.entries()
        for (const [key, value] of entries) {
            const location: any = value.toJSON()
            const token = getTokenFromLocation(location, ethChainId)
            if (!token) {
                continue
            }

            const assetId: any = key.args.at(0)
            const asset: any = (
                await this.provider.query.assetRegistry.currencyMetadatas(assetId)
            ).toPrimitive()

            assets[token] = {
                token,
                name: String(asset.name),
                minimumBalance: BigInt(asset.minimalBalance),
                symbol: String(asset.symbol),
                decimals: Number(asset.decimals),
                isSufficient: false,
            }
        }
        return assets
    }
}
