import { ApiPromise } from "@polkadot/api"
import { Asset, AssetMap, ChainProperties } from "@snowbridge/base-types"
import { PNAMap, SubstrateAccount } from "../assets_v2"
import { Result } from "@polkadot/types"
import { XcmDryRunApiError, XcmDryRunEffects } from "@polkadot/types/interfaces"
import { erc20Location, HERE_LOCATION, parachainLocation } from "../xcmBuilder"

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

    getNativeBalanceLocation(relativeTo: "here" | "sibling"): any {
        switch (relativeTo) {
            case "sibling":
                return parachainLocation(this.parachainId)
            case "here":
                return HERE_LOCATION
        }
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
            console.error(result)
            throw Error(`Can not query XCM Weight.`)
        }
        let dotAsset = undefined
        const assets = result.ok.v4 || result.ok.v5
        for (const asset of assets) {
            if (asset.id.parents === 1 && asset.id.interior.here === null) {
                dotAsset = asset
            }
        }
        let deliveryFee
        if (!dotAsset) {
            console.warn("Could not find DOT in result", result)
            deliveryFee = 0n
        } else {
            deliveryFee = BigInt(dotAsset.fun.fungible.toString())
        }
        return deliveryFee
    }

    async calculateDeliveryFeeInNative(destParachainId: number, xcm: any): Promise<bigint> {
        const result = (
            await this.provider.call.xcmPaymentApi.queryDeliveryFees(
                { v4: { parents: 1, interior: { x1: [{ parachain: destParachainId }] } } },
                xcm
            )
        ).toPrimitive() as any
        if (!result.ok) {
            throw Error(`Can not query XCM Weight.`)
        }
        let nativeAsset = undefined
        const assets = result.ok.v4 || result.ok.v5
        for (const asset of assets) {
            if (asset.id.parents === 0 && asset.id.interior.here === null) {
                nativeAsset = asset
            }
        }

        let deliveryFee
        if (!nativeAsset) {
            console.warn("Could not find NATIVE in result", result)
            deliveryFee = 0n
        } else {
            deliveryFee = BigInt(nativeAsset.fun.fungible.toString())
        }

        return deliveryFee
    }

    async dryRunXcm(originParaId: number, xcm: any, findForwardedDestination?: number) {
        const originLocation = {
            v4: { parents: 1, interior: { x1: [{ parachain: originParaId }] } },
        }

        const result = await this.provider.call.dryRunApi.dryRunXcm<
            Result<XcmDryRunEffects, XcmDryRunApiError>
        >(originLocation, xcm)

        const resultHuman = result.toHuman() as any
        const success = result.isOk && result.asOk.executionResult.isComplete

        let forwardedDestination
        if (!success) {
            console.error(`Error during dry run:`, xcm.toHuman(), result.toHuman())
        } else if (findForwardedDestination) {
            const destinationParaId = findForwardedDestination
            forwardedDestination = result.asOk.forwardedXcms.find((x) => {
                return (
                    x[0].isV4 &&
                    x[0].asV4.parents.toNumber() === 1 &&
                    x[0].asV4.interior.isX1 &&
                    x[0].asV4.interior.asX1[0].isParachain &&
                    x[0].asV4.interior.asX1[0].asParachain.toNumber() === destinationParaId
                )
            })
            if (!forwardedDestination) {
                forwardedDestination = result.asOk.forwardedXcms.find((x) => {
                    return (
                        x[0].isV5 &&
                        x[0].asV5.parents.toNumber() === 1 &&
                        x[0].asV5.interior.isX1 &&
                        x[0].asV5.interior.asX1[0].isParachain &&
                        x[0].asV5.interior.asX1[0].asParachain.toNumber() === destinationParaId
                    )
                })
            }
        }

        return {
            success,
            errorMessage: resultHuman.Ok?.executionResult.Incomplete?.error,
            forwardedDestination,
        }
    }

    abstract getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint>
    abstract getDotBalance(account: string): Promise<bigint>
    abstract getAssets(ethChainId: number, pnas: PNAMap): Promise<AssetMap>
    abstract getXC20DOT(): string | undefined
}
