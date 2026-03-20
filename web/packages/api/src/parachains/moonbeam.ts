import { ApiPromise } from "@polkadot/api"
import { ParachainBase } from "./parachainBase"
import { AssetMap, EthereumProvider, EthereumProviderTypes, PNAMap } from "@snowbridge/base-types"
import { getTokenFromLocation } from "../xcmBuilder"
import { DOT_LOCATION } from "../assets_v2"

const MOONBEAM_ERC20_ABI = [
    "function name() view returns (string)",
    "function symbol() view returns (string)",
    "function decimals() view returns (uint8)",
    "function balanceOf(address) view returns (uint256)",
]

function toMoonbeamXC20(assetId: bigint) {
    const xc20 = assetId.toString(16).toLowerCase()
    return "0xffffffff" + xc20
}

async function getMoonbeamEvmForeignAssetBalanceWithProvider(
    ethereumProvider: EthereumProvider<EthereumProviderTypes>,
    api: ApiPromise,
    token: string,
    account: string,
) {
    const method = "balanceOf"
    const data = ethereumProvider.encodeFunctionData(MOONBEAM_ERC20_ABI, method, [account])
    const result = await api.call.ethereumRuntimeRPCApi.call(
        "0x0000000000000000000000000000000000000000",
        token,
        data,
        0n,
        500_000n,
        null,
        null,
        null,
        false,
        null,
        null,
    )
    const resultJson = result.toPrimitive() as any
    if (!(resultJson?.ok?.exitReason?.succeed === "Returned")) {
        console.error(resultJson)
        throw Error(
            `Could not fetch balance for ${token}: ${JSON.stringify(resultJson?.ok?.exitReason)}`,
        )
    }
    const retVal = ethereumProvider.decodeFunctionResult<bigint | readonly [bigint]>(
        MOONBEAM_ERC20_ABI,
        method,
        resultJson?.ok?.value,
    )
    return BigInt(Array.isArray(retVal) ? retVal[0] : retVal)
}

async function getMoonbeamEvmAssetMetadataWithProvider(
    ethereumProvider: EthereumProvider<EthereumProviderTypes>,
    api: ApiPromise,
    method: string,
    token: string,
) {
    const data = ethereumProvider.encodeFunctionData(MOONBEAM_ERC20_ABI, method, [])
    const result = await api.call.ethereumRuntimeRPCApi.call(
        "0x0000000000000000000000000000000000000000",
        token,
        data,
        0n,
        500_000n,
        null,
        null,
        null,
        false,
        null,
        null,
    )
    const resultJson = result.toPrimitive() as any
    if (!(resultJson?.ok?.exitReason?.succeed === "Returned")) {
        console.error(resultJson)
        throw Error(
            `Could not fetch metadata for ${token}: ${JSON.stringify(resultJson?.ok?.exitReason)}`,
        )
    }
    const retVal = ethereumProvider.decodeFunctionResult<
        string | bigint | readonly [string | bigint]
    >(
        MOONBEAM_ERC20_ABI,
        method,
        resultJson?.ok?.value,
    )
    return Array.isArray(retVal) ? retVal[0] : retVal
}

export class MoonbeamParachain extends ParachainBase {
    #xcDOT?: string
    #ethereumProvider: EthereumProvider<EthereumProviderTypes>

    constructor(
        provider: ApiPromise,
        parachainId: number,
        specName: string,
        specVersion: number,
        ethereumProvider: EthereumProvider<EthereumProviderTypes>,
    ) {
        super(provider, parachainId, specName, specVersion)
        this.#ethereumProvider = ethereumProvider
    }

    getXC20DOT() {
        return this.#xcDOT
    }

    async getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint> {
        // For PNA, use assetId directly; for ENA, query assetId by Multilocation
        let paraAssetId = pnaAssetId
        // If we cannot find the asset in asset manager look in foreign assets.
        if (!paraAssetId) {
            // evmForeignAssets uses xcm v4 so we use the original location.
            paraAssetId = (
                (
                    await this.provider.query.evmForeignAssets.assetsByLocation(location)
                ).toPrimitive() as any
            )[0]
        }

        if (!paraAssetId) {
            throw Error(`Asset not registered for spec ${this.specName}.`)
        }

        const xc20 = toMoonbeamXC20(BigInt(paraAssetId))
        return await getMoonbeamEvmForeignAssetBalanceWithProvider(
            this.#ethereumProvider,
            this.provider,
            xc20,
            account,
        )
    }

    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(DOT_LOCATION, account)
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        const foreignEntries = await this.provider.query.evmForeignAssets.assetsById.entries()
        for (const [key, value] of foreignEntries) {
            const location = value.toJSON() as any

            const assetId = BigInt(key.args[0]?.toPrimitive() as any)
            const xc20 = toMoonbeamXC20(assetId)

            if (location.parents === 1 && location.interior.here !== undefined) {
                this.#xcDOT = xc20
            }

            const token = getTokenFromLocation(location, ethChainId)
            if (!token) {
                continue
            }
            // we found the asset in pallet-assets so we can skip evmForeignAssets.
            if (assets[token]) {
                continue
            }

            const symbol = await getMoonbeamEvmAssetMetadataWithProvider(
                this.#ethereumProvider,
                this.provider,
                "symbol",
                xc20,
            )
            const name = await getMoonbeamEvmAssetMetadataWithProvider(
                this.#ethereumProvider,
                this.provider,
                "name",
                xc20,
            )
            const decimals = await getMoonbeamEvmAssetMetadataWithProvider(
                this.#ethereumProvider,
                this.provider,
                "decimals",
                xc20,
            )
            assets[token] = {
                token,
                name: String(name),
                minimumBalance: 1n,
                symbol: String(symbol),
                decimals: Number(decimals),
                isSufficient: true,
                xc20,
            }
        }
        return assets
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
