import { PNAMap } from "../assets_v2"
import { AssetMap } from "@snowbridge/base-types"
import { ParachainBase } from "./parachainBase"
import { convertToXcmV3X1, getTokenFromLocation } from "../xcmBuilder"

export class AcalaParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint> {
        let paraAssetId = pnaAssetId
        const v3location = convertToXcmV3X1(location)
        if (!paraAssetId) {
            paraAssetId = (
                await this.provider.query.assetRegistry.locationToCurrencyIds(v3location)
            ).toPrimitive()
        }
        if (!paraAssetId) {
            throw Error(`'${JSON.stringify(v3location)}' not registered for spec ${this.specName}.`)
        }
        const accountData = (
            await this.provider.query.tokens.accounts(account, paraAssetId)
        ).toPrimitive() as any
        return BigInt(accountData?.free ?? 0n)
    }

    async getDotBalance(account: string): Promise<bigint> {
        const accountData = (
            await this.provider.query.tokens.accounts(account, { Token: "DOT" })
        ).toPrimitive() as any
        return BigInt(accountData.free)
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        const entries = await this.provider.query.assetRegistry.foreignAssetLocations.entries()
        for (const [value, key] of entries) {
            const location: any = key.toPrimitive()
            const token = getTokenFromLocation(location, ethChainId)
            if (!token) {
                continue
            }

            const assetId: any = value.args[0]?.toPrimitive()
            const asset: any = (
                await this.provider.query.assetRegistry.assetMetadatas({ foreignAssetId: assetId })
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

    // Acala does not support xcm fee payment queries
    async calculateXcmFee(_destinationXcm: any, asset: any): Promise<bigint> {
        console.warn(
            `${this.specName} does not support calculating fee with asset '${JSON.stringify(
                asset
            )}'. Using default.`
        )

        return 300_000_000n
    }

    async calculateDeliveryFeeInDOT(_destParachainId: number, _xcm: any): Promise<bigint> {
        throw Error(`${this.specName} does not support.`)
    }
}
