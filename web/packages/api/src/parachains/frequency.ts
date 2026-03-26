import { AssetMap, PNAMap } from "@snowbridge/base-types"
import { ParachainBase } from "./parachainBase"
import { DOT_LOCATION, HERE_LOCATION, WESTEND_GENESIS } from "../xcmBuilder"

export const FREQUENCY_WESTEND_TOKEN_ID = "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f"
export const FREQUENCY_WESTEND_PARA_ID = 2313
export const FREQUENCY_TOKEN_LOCATION_ON_WESTEND_AH = {
    parents: 1,
    interior: { x1: [{ parachain: FREQUENCY_WESTEND_PARA_ID }] },
}
export const FREQUENCY_WESTEND_TOKEN_LOCATION_ON_ETHEREUM = {
    parents: 1,
    interior: {
        x2: [
            {
                globalConsensus: {
                    byGenesis: WESTEND_GENESIS,
                },
            },
            { parachain: FREQUENCY_WESTEND_PARA_ID },
        ],
    },
}

export class FrequencyParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        let accountData = (
            await this.provider.query.foreignAssets.account(location, account)
        ).toPrimitive() as any

        return BigInt(accountData?.balance ?? 0n)
    }

    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(DOT_LOCATION, account)
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        if (this.specName === "frequency-testnet") {
            assets[FREQUENCY_WESTEND_TOKEN_ID.toLowerCase()] = {
                token: FREQUENCY_WESTEND_TOKEN_ID.toLowerCase(),
                name: "XRQCY",
                minimumBalance: 1_000_000n,
                symbol: "XRQCY",
                decimals: 8,
                isSufficient: true,
                location: HERE_LOCATION,
                locationOnAH: FREQUENCY_TOKEN_LOCATION_ON_WESTEND_AH,
                locationOnEthereum: FREQUENCY_WESTEND_TOKEN_LOCATION_ON_ETHEREUM,
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

    async swapAsset1ForAsset2(
        _asset1: any,
        _asset2: any,
        _exactAsset1Balance: bigint,
    ): Promise<bigint> {
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
