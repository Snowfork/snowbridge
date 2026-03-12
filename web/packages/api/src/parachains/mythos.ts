import { DOT_LOCATION, erc20Location } from "../xcmBuilder"
import { AssetMap, PNAMap } from "@snowbridge/base-types"
import { ParachainBase } from "./parachainBase"

export const MUSE_CHAIN_ID = 11155111 // Sepolia
export const MUSE_TOKEN_ID = "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee"
export const MYTHOS_CHAIN_ID = 1 // Ethereum Mainnet
export const MYTHOS_TOKEN_ID = "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003"

export class MythosParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        if (
            this.specName === "muse" &&
            JSON.stringify(location) == JSON.stringify(erc20Location(MUSE_CHAIN_ID, MUSE_TOKEN_ID))
        ) {
            return await this.getNativeBalance(account, true)
        } else if (
            this.specName === "mythos" &&
            JSON.stringify(location) ==
                JSON.stringify(erc20Location(MYTHOS_CHAIN_ID, MYTHOS_TOKEN_ID))
        ) {
            return await this.getNativeBalance(account, true)
        } else {
            throw Error(
                `Cannot get balance for spec ${this.specName}. Location = ${JSON.stringify(
                    location,
                )}`,
            )
        }
    }

    getDotBalance(_account: string): Promise<bigint> {
        throw Error(`Cannot get DOT balance for spec ${this.specName}.`)
    }

    async getAssets(_ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        if (this.specName === "muse") {
            assets[MUSE_TOKEN_ID.toLowerCase()] = {
                token: MUSE_TOKEN_ID.toLowerCase(),
                name: "Muse",
                minimumBalance: 10_000_000_000_000_000n,
                symbol: "MUSE",
                decimals: 18,
                isSufficient: true,
            }
        } else if (this.specName === "mythos") {
            assets[MYTHOS_TOKEN_ID.toLowerCase()] = {
                token: MYTHOS_TOKEN_ID.toLowerCase(),
                name: "Mythos",
                minimumBalance: 10_000_000_000_000_000n,
                symbol: "MYTH",
                decimals: 18,
                isSufficient: true,
            }
        } else {
            throw Error(
                `Cannot get balance for spec ${this.specName}. Location = ${JSON.stringify(
                    location,
                )}`,
            )
        }
        return assets
    }

    async calculateXcmFee(destinationXcm: any, asset: any): Promise<bigint> {
        if (JSON.stringify(asset) == JSON.stringify(DOT_LOCATION)) {
            console.warn(
                `${this.specName} does not support calculating fee for asset '${JSON.stringify(
                    asset,
                )}'. Using default.`,
            )
            return 1_000_000_000n
        }
        return await this.calculateXcmFee(destinationXcm, asset)
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
