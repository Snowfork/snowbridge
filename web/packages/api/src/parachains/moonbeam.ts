import { ApiPromise } from "@polkadot/api"
import { ethers } from "ethers"
import { ParachainBase } from "./parachainBase"
import { AssetMap, PNAMap } from "../assets_v2"
import { convertToXcmV3X1, DOT_LOCATION, getTokenFromLocation } from "../xcmBuilder"

const MOONBEAM_ERC20_ABI = [
    "function name() view returns (string)",
    "function symbol() view returns (string)",
    "function decimals() view returns (uint8)",
    "function balanceOf(address) view returns (uint256)",
]
const MOONBEAM_ERC20 = new ethers.Interface(MOONBEAM_ERC20_ABI)

export function toMoonbeamXC20(assetId: bigint) {
    const xc20 = assetId.toString(16).toLowerCase()
    return "0xffffffff" + xc20
}

export async function getMoonbeamLocationBalance(
    pnaAssetId: any,
    location: any,
    provider: ApiPromise,
    specName: string,
    account: string
) {
    // For PNA, use assetId directly; for ENA, query assetId by Multilocation
    let paraAssetId = pnaAssetId
    // If we cannot find the asset in asset manager look in foreign assets.
    if (!paraAssetId) {
        // evmForeignAssets uses xcm v4 so we use the original location.
        paraAssetId = (
            (await provider.query.evmForeignAssets.assetsByLocation(location)).toPrimitive() as any
        )[0]
    }

    if (!paraAssetId) {
        throw Error(`Asset not registered for spec ${specName}.`)
    }

    const xc20 = toMoonbeamXC20(BigInt(paraAssetId))
    return await getMoonbeamEvmForeignAssetBalance(provider, xc20, account)
}

export async function getMoonbeamEvmForeignAssetBalance(
    api: ApiPromise,
    token: string,
    account: string
) {
    const method = "balanceOf"
    const data = MOONBEAM_ERC20.encodeFunctionData(method, [account])
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
        null
    )
    const resultJson = result.toPrimitive() as any
    if (!(resultJson?.ok?.exitReason?.succeed === "Returned")) {
        console.error(resultJson)
        throw Error(
            `Could not fetch balance for ${token}: ${JSON.stringify(resultJson?.ok?.exitReason)}`
        )
    }
    const retVal = MOONBEAM_ERC20.decodeFunctionResult(method, resultJson?.ok?.value)
    return BigInt(retVal[0])
}

export async function getMoonbeamEvmAssetMetadata(api: ApiPromise, method: string, token: string) {
    const data = MOONBEAM_ERC20.encodeFunctionData(method, [])
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
        null
    )
    const resultJson = result.toPrimitive() as any
    if (!(resultJson?.ok?.exitReason?.succeed === "Returned")) {
        console.error(resultJson)
        throw Error(
            `Could not fetch metadata for ${token}: ${JSON.stringify(resultJson?.ok?.exitReason)}`
        )
    }
    const retVal = MOONBEAM_ERC20.decodeFunctionResult(method, resultJson?.ok?.value)
    return retVal[0]
}

export class MoonbeamParachain extends ParachainBase {
    #xcDOT?: string

    getXC20DOT() {
        return this.#xcDOT
    }

    async getLocationBalance(location: any, account: string, pnaAssetId?: any): Promise<bigint> {
        let paraAssetId = pnaAssetId
        if (!paraAssetId) {
            // Moonbeam only supports v3 xcm locations on asset Manager. Deep clone the location because
            // we might modify it.
            const assetManagerLocation = convertToXcmV3X1(location)
            paraAssetId = (
                await this.provider.query.assetManager.assetTypeId({ xcm: assetManagerLocation })
            ).toPrimitive()
        }

        // If we cannot find the asset in asset manager look in foreign assets.
        if (!paraAssetId) {
            // evmForeignAssets uses xcm v4 so we use the original location.
            paraAssetId = (
                (
                    await this.provider.query.evmForeignAssets.assetsByLocation(location)
                ).toPrimitive() as any
            )[0]
            const xc20 = toMoonbeamXC20(BigInt(paraAssetId))
            return await getMoonbeamEvmForeignAssetBalance(this.provider, xc20, account)
        }

        if (!paraAssetId) {
            throw Error(`Asset not registered for spec ${this.specName}.`)
        }

        const accountData = (
            await this.provider.query.assets.account(paraAssetId, account)
        ).toPrimitive() as any
        return BigInt(accountData?.balance ?? 0n)
    }

    getDotBalance(account: string): Promise<bigint> {
        return this.getLocationBalance(DOT_LOCATION, account)
    }

    async getAssets(ethChainId: number, _pnas: PNAMap): Promise<AssetMap> {
        const assets: AssetMap = {}
        let xcDOT: string | undefined
        const foreignEntries = await this.provider.query.evmForeignAssets.assetsById.entries()
        for (const [key, value] of foreignEntries) {
            const location = value.toJSON() as any

            const assetId = BigInt(key.args.at(0)?.toPrimitive() as any)
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

            const symbol = await getMoonbeamEvmAssetMetadata(this.provider, "symbol", xc20)
            const name = await getMoonbeamEvmAssetMetadata(this.provider, "name", xc20)
            const decimals = await getMoonbeamEvmAssetMetadata(this.provider, "decimals", xc20)
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
}
