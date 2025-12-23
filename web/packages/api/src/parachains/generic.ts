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
        return this.getNativeBalance(account)
    }
    getAssets(_ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        throw new Error("Method not implemented.")
    }
}
