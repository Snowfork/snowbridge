import { AssetMap } from "src/assets_v2";
import { ParachainBase } from "./parachain";
import { getTokenFromLocation } from "src/xcmBuilder";

export class AssetHubParachain extends ParachainBase {
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
        return this.getNativeBalance(account)
    }

    async getAssets(ethChainId: number): Promise<AssetMap> {
        const assets: AssetMap = {}
        // ERC20
        const entries = await this.provider.query.foreignAssets.asset.entries()
        for (const [key, value] of entries) {
            const location: any = key.args.at(0)?.toJSON()
            if (!location) {
                console.warn(`Could not convert ${key.toHuman()} to location for ${this.specName}.`)
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
        // PNA
        return assets
    }
}