import { AssetMap, PNAMap } from "@snowbridge/base-types"
import { ParachainBase } from "./parachainBase"
import { getTokenFromLocation } from "../xcmBuilder"
import { DOT_LOCATION } from "../assets_v2"

const HDX_LOCATION = { parents: 0, interior: { x1: [{ generalIndex: 0 }] } }
const HDX_LOCATION_ON_ASSET_HUB = {
    parents: 1,
    interior: { x2: [{ parachain: 2034 }, { generalIndex: 0 }] },
}

export class HydrationParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    getMaxWeight(): { refTime: bigint; proofSize: bigint } {
        return { refTime: 20_000_000_000n, proofSize: 500_000n }
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        const paraAssetId = (
            await this.provider.query.assetRegistry.locationAssets(location)
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

    async getAssets(ethChainId: number, pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        const entries = await this.provider.query.assetRegistry.assetLocations.entries()
        for (const [id, value] of entries) {
            const location: any = value.toJSON()
            const token = getTokenFromLocation(location, ethChainId)
            if (!token) {
                continue
            }

            const assetId = Number(id.args[0]?.toString())
            const lockdownState: any =
                await this.provider.query.circuitBreaker.assetLockdownState(assetId)
            if (!lockdownState.isSome || !lockdownState.unwrap().isUnlocked) {
                continue
            }

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

        // HDX is Hydration's native asset and is identified by GeneralIndex(0).
        const chainInfo = await this.chainProperties()
        const existentialDeposit = BigInt(
            this.provider.consts.balances.existentialDeposit.toPrimitive() as any,
        )
        for (const { token, foreignId, ethereumlocation } of Object.values(pnas)) {
            if (!isHydrationNativeLocation(ethereumlocation, this.parachainId)) {
                continue
            }

            assets[token.toLowerCase()] = {
                token: token.toLowerCase(),
                name: String(chainInfo.name),
                minimumBalance: existentialDeposit,
                symbol: chainInfo.tokenSymbols,
                decimals: chainInfo.tokenDecimals,
                isSufficient: true,
                location: HDX_LOCATION,
                locationOnAH: HDX_LOCATION_ON_ASSET_HUB,
                locationOnEthereum: ethereumlocation,
                foreignId,
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

function isHydrationNativeLocation(location: any, parachainId: number): boolean {
    return (
        location?.parents === 1 &&
        location.interior?.x3?.[0]?.globalConsensus?.polkadot !== undefined &&
        location.interior.x3[1]?.parachain === parachainId &&
        location.interior.x3[2]?.generalIndex === 0
    )
}
