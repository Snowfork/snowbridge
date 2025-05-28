import { PNAMap, AssetMap } from "../assets_v2";
import { AssetHubParachain } from "./assetHub"
import { dotLocationOnKusamaAssetHub } from "../xcmBuilder"

export class AssetHubKusamaParachain extends AssetHubParachain {
    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(provider, dotLocationOnKusamaAssetHub, account)
    }
}