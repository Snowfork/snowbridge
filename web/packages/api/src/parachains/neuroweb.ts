import { ParachainBase } from "./parachainBase"
import { DOT_LOCATION, erc20Location } from "../xcmBuilder"
import { AssetMap, PNAMap } from "@snowbridge/base-types"
import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { ISubmittableResult } from "@polkadot/types/types"

export const NEUROWEB_TEST_CHAIN_ID = 11155111 // Sepolia
export const NEUROWEB_TEST_TOKEN_ID = "0xef32abea56beff54f61da319a7311098d6fbcea9"
export const NEUROWEB_CHAIN_ID = 1 // Ethereum Mainnet
export const NEUROWEB_TOKEN_ID = "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f"
const TRAC_ASSET_ID = 1

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

    async getDotBalance(account: string): Promise<bigint> {
        // Neuro web supports DOT but we disable it to allow fee payment in Neuro
        // const accountData = (
        //     await this.provider.query.foreignAssets.account(DOT_LOCATION, account)
        // ).toPrimitive() as any
        // return BigInt(accountData?.balance ?? 0n)

        throw Error(`Spec ${this.specName} supports DOT but is disabled.`)
    }

    getNativeBalanceLocation(relativeTo: "here" | "sibling"): any {
        switch (relativeTo) {
            case "here":
                return {
                    parents: 0,
                    interior: { x1: [{ palletInstance: 10 }] },
                }
            case "sibling":
                return {
                    parents: 1,
                    interior: { x2: [{ parachain: this.parachainId }, { palletInstance: 10 }] },
                }
        }
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        if (this.specName !== "origintrail-parachain") {
            throw Error(
                `Cannot get balance for spec ${this.specName}. Location = ${JSON.stringify(
                    location,
                )}`,
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

    async wrapExecutionFeeInNative(parachain: ApiPromise) {
        // Mock transaction to get extrinsic fee
        let tx = parachain.tx.wrapper.tracWrap(100000000)
        const paymentInfo = await tx.paymentInfo(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        const executionFee = paymentInfo["partialFee"].toBigInt()
        console.log("wrap execution fee:", executionFee)
        return executionFee
    }

    async unwrapExecutionFeeInNative(parachain: ApiPromise) {
        // Mock transaction to get extrinsic fee
        let tx = parachain.tx.wrapper.tracUnwrap(100000000)
        const paymentInfo = await tx.paymentInfo(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        const executionFee = paymentInfo["partialFee"].toBigInt()
        console.log("unwrap execution fee:", executionFee)
        return executionFee
    }

    snowTRACBalance(account: string, ethChainId: number) {
        if (ethChainId === NEUROWEB_TEST_CHAIN_ID) {
            return this.getLocationBalance(
                erc20Location(ethChainId, NEUROWEB_TEST_TOKEN_ID),
                account,
            )
        } else if (ethChainId === NEUROWEB_CHAIN_ID) {
            return this.getLocationBalance(erc20Location(ethChainId, NEUROWEB_TOKEN_ID), account)
        } else {
            throw Error(`Cannot get snowTRAC balance for chain ID ${ethChainId}.`)
        }
    }

    async tracBalance(account: string) {
        const accountData = (
            await this.provider.query.assets.account(TRAC_ASSET_ID, account)
        ).toPrimitive() as any
        return BigInt(accountData?.balance ?? 0n)
    }

    createWrapTx(amount: bigint): SubmittableExtrinsic<"promise", ISubmittableResult> {
        // TODO: Delete, unused
        return this.provider.tx.wrapper.tracWrap(amount)
    }

    createUnwrapTx(amount: bigint): SubmittableExtrinsic<"promise", ISubmittableResult> {
        // TODO: Delete, unused
        return this.provider.tx.wrapper.tracUnwrap(amount)
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
