import { ApiPromise } from "@polkadot/api"
import { ethers } from "ethers"

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

export function convertToXcmV3X1(location: any) {
    if (location.interior.x1) {
        const convertedLocation = JSON.parse(JSON.stringify(location))
        convertedLocation.interior.x1 = convertedLocation.interior.x1[0]
        return convertedLocation
    }
    return location
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

async function getMoonbeamEvmForeignAssetBalance(api: ApiPromise, token: string, account: string) {
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
