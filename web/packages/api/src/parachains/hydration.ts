import { PNAMap } from "../assets_v2"
import { AssetMap } from "@snowbridge/base-types"
import { ParachainBase } from "./parachainBase"
import { convertToXcmV3X1, DOT_LOCATION, getTokenFromLocation } from "../xcmBuilder"

export class HydrationParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        const paraAssetId = (
            await this.provider.query.assetRegistry.locationAssets(convertToXcmV3X1(location))
        ).toPrimitive()
        if (!paraAssetId) {
            throw Error(`DOT not registered for spec ${this.specName}.`)
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
        const entries = await this.provider.query.assetRegistry.assetLocations.entries()
        for (const [id, value] of entries) {
            const location: any = value.toJSON()
            const token = getTokenFromLocation(location, ethChainId)
            if (!token) {
                continue
            }

            const assetId = Number(id.args.at(0)?.toString())
            const asset: any = (
                await this.provider.query.assetRegistry.assets(assetId)
            ).toPrimitive()

            assets[token] = {
                token,
                name: String(asset.name ?? ""),
                minimumBalance: BigInt(asset.existentialDeposit),
                symbol: String(asset.symbol ?? ""),
                decimals: Number(asset.decimals),
                isSufficient: Boolean(asset.isSufficient),
            }
        }
        return assets
    }
}
