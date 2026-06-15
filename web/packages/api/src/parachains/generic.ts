import { AssetMap, PNAMap } from "@snowbridge/base-types"
import { ParachainBase } from "./parachainBase"

export class GenericChain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint> {
        throw new Error("Method not implemented.")
    }
    getDotBalance(account: string): Promise<bigint> {
        return this.getNativeBalance(account, true)
    }
    getAssets(_ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        throw new Error("Method not implemented.")
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
