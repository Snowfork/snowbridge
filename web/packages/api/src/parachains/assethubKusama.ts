import { PNAMap, AssetMap } from "../assets_v2";
import { dotLocationOnKusamaAssetHub } from "../xcmBuilder"
import { AssetHubParachain } from "./assethub";

export class AssetHubKusamaParachain extends AssetHubParachain {
    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(dotLocationOnKusamaAssetHub, account)
    }
}
