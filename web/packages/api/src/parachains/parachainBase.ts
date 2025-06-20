import { ApiPromise } from "@polkadot/api"
import { Asset, AssetMap, ChainProperties, PNAMap, SubstrateAccount } from "../assets_v2"
import { erc20Location } from "../xcmBuilder"

export abstract class ParachainBase {
    provider: ApiPromise
    specName: string
    specVersion: number
    parachainId: number
    #chainProperties?: ChainProperties

    constructor(provider: ApiPromise, parachainId: number, specName: string, specVersion: number) {
        this.provider = provider
        this.specName = specName
        this.specVersion = specVersion
        this.parachainId = parachainId
    }

    async chainProperties(): Promise<ChainProperties> {
        if (this.#chainProperties) return this.#chainProperties

        const [properties, name] = await Promise.all([
            this.provider.rpc.system.properties(),
            this.provider.rpc.system.chain(),
        ])
        const tokenSymbols = properties.tokenSymbol.unwrapOrDefault().at(0)?.toString()
        const tokenDecimals = properties.tokenDecimals.unwrapOrDefault().at(0)?.toNumber()
        const isEthereum = properties.isEthereum.toPrimitive()
        const ss58Format =
            (this.provider.consts.system.ss58Prefix.toPrimitive() as number) ??
            properties.ss58Format.unwrapOr(null)?.toNumber()
        const accountType = this.provider.registry.getDefinition("AccountId")

        let evmChainId: number | undefined
        if (this.provider.query.evmChainId) {
            evmChainId = Number((await this.provider.query.evmChainId.chainId()).toPrimitive())
        } else if (this.provider.query.ethereumChainId) {
            evmChainId = Number((await this.provider.query.ethereumChainId.chainId()).toPrimitive())
        } else {
            evmChainId = undefined
        }

        if (accountType !== "AccountId20" && accountType !== "AccountId32") {
            throw Error(`Unknown account type ${accountType} for runtime ${this.specName}.`)
        }
        this.#chainProperties = {
            tokenSymbols: String(tokenSymbols),
            tokenDecimals: Number(tokenDecimals),
            ss58Format,
            isEthereum,
            accountType,
            evmChainId,
            name: name.toPrimitive(),
            specName: this.specName,
            specVersion: this.specVersion,
        }
        return this.#chainProperties
    }

    async getNativeAccount(account: string): Promise<SubstrateAccount> {
        const accountData = (await this.provider.query.system.account(account)).toPrimitive() as any
        return {
            nonce: BigInt(accountData.nonce),
            consumers: BigInt(accountData.consumers),
            providers: BigInt(accountData.providers),
            sufficients: BigInt(accountData.sufficients),
            data: {
                free: BigInt(accountData.data.free),
                reserved: BigInt(accountData.data.reserved),
                frozen: BigInt(accountData.data.frozen),
            },
        }
    }

    async getNativeBalance(account: string): Promise<bigint> {
        const acc = await this.getNativeAccount(account)
        return acc.data.free
    }

    getTokenBalance(
        account: string,
        ethChainId: number,
        tokenAddress: string,
        asset?: Asset
    ): Promise<bigint> {
        return this.getLocationBalance(
            asset?.location ?? erc20Location(ethChainId, tokenAddress),
            account,
            asset?.assetId
        )
    }

    async calculateXcmFee(destinationXcm: any, asset: any): Promise<bigint> {
        const weight = (
            await this.provider.call.xcmPaymentApi.queryXcmWeight(destinationXcm)
        ).toPrimitive() as any
        if (!weight.ok) {
            throw Error(`Can not query XCM Weight.`)
        }

        let feeInDot: any
        feeInDot = (
            await this.provider.call.xcmPaymentApi.queryWeightToAssetFee(weight.ok, {
                v4: asset,
            })
        ).toPrimitive() as any
        // For compatibility with Westend, which has XCMV5 enabled.
        if (!feeInDot.ok) {
            feeInDot = (
                await this.provider.call.xcmPaymentApi.queryWeightToAssetFee(weight.ok, {
                    v5: asset,
                })
            ).toPrimitive() as any
            if (!feeInDot.ok) throw Error(`Can not convert weight to fee in DOT.`)
        }
        const executionFee = BigInt(feeInDot.ok.toString())

        return executionFee
    }

    async calculateDeliveryFeeInDOT(destParachainId: number, xcm: any): Promise<bigint> {
        const result = (
            await this.provider.call.xcmPaymentApi.queryDeliveryFees(
                { v4: { parents: 1, interior: { x1: [{ parachain: destParachainId }] } } },
                xcm
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
            console.info("Could not find DOT in result", result)
            throw Error(`Can not query XCM Weight.`)
        }

        const deliveryFee = BigInt(dotAsset.fun.fungible.toString())

        return deliveryFee
    }

    async getConversationPalletSwap(
        asset1: any,
        asset2: any,
        exactAsset2Balance: bigint
    ): Promise<bigint> {
        const result = await this.provider.call.assetConversionApi.quotePriceTokensForExactTokens(
            asset1,
            asset2,
            exactAsset2Balance,
            true
        )
        const asset1Balance = result.toPrimitive() as any
        if (asset1Balance == null) {
            throw Error(
                `No pool set up in asset conversion pallet for '${JSON.stringify(
                    asset1
                )}' and '${JSON.stringify(asset2)}'.`
            )
        }
        return BigInt(asset1Balance)
    }

    abstract getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint>
    abstract getDotBalance(account: string): Promise<bigint>
    abstract getAssets(ethChainId: number, pnas: PNAMap): Promise<AssetMap>
    abstract getXC20DOT(): string | undefined
}
