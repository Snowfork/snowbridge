import {ParachainBase} from "./parachainBase";
import {DOT_LOCATION, erc20Location} from "../xcmBuilder";
import {PNAMap} from "../assets_v2";
import {AssetMap} from "@snowbridge/base-types";

export const NEUROWEB_TEST_CHAIN_ID = 11155111 // Sepolia
export const NEUROWEB_TEST_TOKEN_ID = "0xef32abea56beff54f61da319a7311098d6fbcea9" // TODO
export const NEUROWEB_CHAIN_ID = 1 // Ethereum Mainnet
export const NEUROWEB_TOKEN_ID = "" // TODO

export class NeurowebParachain extends ParachainBase {
    getXC20DOT() {
        return undefined
    }

    async getLocationBalance(location: any, account: string, _pnaAssetId?: any): Promise<bigint> {
        if (
            this.specName === "origintrail-parachain" &&
            JSON.stringify(location) == JSON.stringify(erc20Location(NEUROWEB_TEST_CHAIN_ID, NEUROWEB_TEST_TOKEN_ID))
        ) {
            return await this.getNativeBalance(account)
        } else if (
            this.specName === "origintrail-parachain" &&
            JSON.stringify(location) == JSON.stringify(erc20Location(NEUROWEB_CHAIN_ID, NEUROWEB_TOKEN_ID))
        ){
            return await this.getNativeBalance(account)
        }
        else {
            throw Error(
                `Cannot get balance for spec ${this.specName}. Location = ${JSON.stringify(
                    location
                )}`
            )
        }
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
                minimumBalance: 10_000_000_000_000_000n,
                symbol: "TRAC",
                decimals: 18,
                isSufficient: true,
            }
        } else if (ethChainId === NEUROWEB_CHAIN_ID) {
            assets[NEUROWEB_TOKEN_ID.toLowerCase()] = {
                token: NEUROWEB_TOKEN_ID.toLowerCase(),
                name: "Trac",
                minimumBalance: 10_000_000_000_000_000n,
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
            return 200_000_000_000n // TODO
        }
        return await this.calculateXcmFee(destinationXcm, asset)
    }
}

