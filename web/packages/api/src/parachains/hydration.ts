import { AssetMap, PNAMap } from "@snowbridge/base-types"
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

            const assetId = Number(id.args[0]?.toString())
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

    async calculateDeliveryFeeInDOT(destParachainId: number, xcm: any): Promise<bigint> {
        const result = (
            await this.provider.call.xcmPaymentApi.queryDeliveryFees(
                { v4: { parents: 1, interior: { x1: [{ parachain: destParachainId }] } } },
                xcm,
            )
        ).toPrimitive() as any
        if (!result.ok) {
            throw Error(`Can not query XCM Weight.`)
        }
        let dotAsset = undefined
        const assets = result.ok.v4 || result.ok.v5
        for (const asset of assets) {
            if (asset.id.parents === 1 && asset.id.interior.here === null) {
                dotAsset = asset
            }
        }
        if (!dotAsset) {
            console.warn(
                "Could not find DOT in result",
                result,
                "using 0 as delivery fee. Dry run will fail if this is incorrect.",
            )
            return 0n
        }
        const deliveryFee = BigInt(dotAsset.fun.fungible.toString())
        return deliveryFee
    }

    swapAsset1ForAsset2(_asset1: any, _asset2: any, _exactAsset1Balance: bigint): Promise<bigint> {
        throw Error(`${this.specName} does not support.`)
    }

    getAssetHubConversionPalletSwap(
        asset1: any,
        asset2: any,
        exactAsset2Balance: bigint,
    ): Promise<bigint> {
        throw Error(`${this.specName} does not support.`)
    }
}
