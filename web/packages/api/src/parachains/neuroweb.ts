import { ParachainBase } from "./parachainBase"
import { DOT_LOCATION, erc20Location } from "../xcmBuilder"
import { PNAMap } from "../assets_v2"
import { AssetMap } from "@snowbridge/base-types"
import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"

export const NEUROWEB_TEST_CHAIN_ID = 11155111 // Sepolia
export const NEUROWEB_TEST_TOKEN_ID = "0xef32abea56beff54f61da319a7311098d6fbcea9"
export const NEUROWEB_CHAIN_ID = 1 // Ethereum Mainnet
export const NEUROWEB_TOKEN_ID = "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f"

export class NeurowebParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        const accountData = (
            await this.provider.query.foreignAssets.account(location, account)
        ).toPrimitive() as any
        return BigInt(accountData?.balance ?? 0n)
    }

    getDotBalance(_account: string): Promise<bigint> {
        throw Error(`Cannot get DOT balance for spec ${this.specName}.`)
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        if (this.specName !== "origintrail-parachain") {
            throw Error(
                `Cannot get balance for spec ${this.specName}. Location = ${JSON.stringify(
                    location
                )}`
            )
        }

        if (ethChainId === NEUROWEB_TEST_CHAIN_ID) {
            assets[NEUROWEB_TEST_TOKEN_ID.toLowerCase()] = {
                token: NEUROWEB_TEST_TOKEN_ID.toLowerCase(),
                name: "Trac",
                minimumBalance: 1_000_000_000_000_000n,
                symbol: "TRAC",
                decimals: 18,
                isSufficient: true,
            }
        } else if (ethChainId === NEUROWEB_CHAIN_ID) {
            assets[NEUROWEB_TOKEN_ID.toLowerCase()] = {
                token: NEUROWEB_TOKEN_ID.toLowerCase(),
                name: "Trac",
                minimumBalance: 1_000_000_000_000_000n,
                symbol: "TRAC",
                decimals: 18,
                isSufficient: true,
            }
        } else {
            throw Error(`Cannot get assets for chain ID ${ethChainId}.`)
        }
        return assets
    }

    async calculateXcmFee(destinationXcm: any, asset: any): Promise<bigint> {
        if (JSON.stringify(asset) == JSON.stringify(DOT_LOCATION)) {
            console.warn(
                `${this.specName} does not support calculating fee for asset '${JSON.stringify(
                    asset
                )}'. Using default.`
            )
            return 14_742_750_000n // TODO10,000,000
        }
        return await this.calculateXcmFee(destinationXcm, asset)
    }
}
